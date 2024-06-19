mod imp;

use std::fs::rename;

use glib::{Cast, IsA};
use gtk::{
    glib,
    prelude::{GtkListStoreExtManual, TreeModelExt, TreeSortableExtManual, TreeViewExt},
    ListStore, SortColumn, SortType, TreeIter, TreePath, TreeView, TreeViewColumn,
};
use regex::Regex;

use crate::{backends::Columns, category::Category};

glib::wrapper! {
pub struct FileListView(ObjectSubclass<imp::FileListViewImp>)
    @extends gtk::Container, gtk::Widget, gtk::TreeView, gtk::Scrollable;
}

impl FileListView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for FileListView {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
#[repr(i32)]
pub enum Direction {
    Up = 0,
    Down,
}

#[derive(Debug)]
#[repr(i32)]
pub enum Filter {
    None = 0,
    Image,
    Favorite,
}

// TODO: move to trait or new store type
fn model_filename(model: &ListStore, iter: &TreeIter) -> String {
    model
        .value(iter, Columns::Name as i32)
        .get::<String>()
        .unwrap_or_default()
}

fn model_category(model: &ListStore, iter: &TreeIter) -> u32 {
    model
        .value(iter, Columns::Cat as i32)
        .get::<u32>()
        .unwrap_or(Category::Unsupported.id())
}

pub trait FileListViewExt: IsA<FileListView> + IsA<TreeView> + 'static {
    fn goto_first(&self);
    fn goto(&self, filename: &str) -> bool;
    fn iter(&self) -> Option<(ListStore, TreeIter)>;
    fn current_filename(&self) -> Option<String>;
    fn navigate(&self, direction: Direction, filter: Filter, count: i32) -> bool;
    fn favorite(&self, directory: &str, direction: Direction) -> bool;
    fn set_sort_column(&self, sort_column_id: SortColumn, order: SortType);
    fn set_unsorted(&self);
}

impl<O: IsA<FileListView> + IsA<TreeView>> FileListViewExt for O {
    fn goto_first(&self) {
        let tp = TreePath::from_indicesv(&[0]);
        self.set_cursor(&tp, None::<&TreeViewColumn>, false);
    }

    fn iter(&self) -> Option<(ListStore, TreeIter)> {
        let (tp, _) = self.cursor();
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(path) = tp {
            model.iter(&path).map(|iter| (model, iter))
        } else {
            model.iter_first().map(|iter| (model, iter))
        }
    }

    fn current_filename(&self) -> Option<String> {
        if let Some((model, iter)) = self.iter() {
            Some(model_filename(&model, &iter))
        } else {
            None
        }
    }

    fn goto(&self, filename: &str) -> bool {
        println!("Goto {filename}");
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(iter) = model.iter_first() {
            loop {
                if filename == model_filename(&model, &iter) {
                    let tp = model.path(&iter).unwrap_or_default();
                    self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                    return true;
                }
                if !model.iter_next(&iter) {
                    break;
                }
            }
        }
        false
    }

    fn navigate(&self, direction: Direction, filter: Filter, count: i32) -> bool {
        if let Some((model, iter)) = self.iter() {
            let mut cnt = count;
            loop {
                let last = iter;
                let result = if matches!(direction, Direction::Up) {
                    model.iter_previous(&iter)
                } else {
                    model.iter_next(&iter)
                };
                if !result {
                    if count != cnt {
                        let tp = model.path(&last).unwrap_or_default();
                        self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                    }
                    return false;
                }

                let cat = model_category(&model, &iter);

                let skip = match filter {
                    Filter::None => false,
                    Filter::Image => cat != Category::Image.id() && cat != Category::Favorite.id(),
                    Filter::Favorite => cat != Category::Favorite.id(),
                };

                if skip {
                    continue;
                }

                cnt -= 1;
                if cnt == 0 {
                    break;
                }
            }
            let tp = model.path(&iter).unwrap_or_default();
            self.set_cursor(&tp, None::<&TreeViewColumn>, false);
            true
        } else {
            false
        }
    }

    fn favorite(&self, directory: &str, direction: Direction) -> bool {
        if let Some((model, iter)) = self.iter() {
            let cat = model_category(&model, &iter);
            if cat != Category::Image.id()
                && cat != Category::Favorite.id()
                && cat != Category::Trash.id()
            {
                return false;
            }

            let filename = model_filename(&model, &iter);
            let re = Regex::new(r"\.([^\.]+)$").unwrap();
            let (new_filename, new_cat) = if matches!(direction, Direction::Up) {
                if filename.contains(".hi.") {
                    return false;
                } else if filename.contains(".lo.") {
                    (filename.replace(".lo", ""), Category::Image)
                } else {
                    (
                        re.replace(&filename, ".hi.$1").to_string(),
                        Category::Favorite,
                    )
                }
            } else if filename.contains(".lo.") {
                return false;
            } else if filename.contains(".hi.") {
                (filename.replace(".hi", ""), Category::Image)
            } else {
                (re.replace(&filename, ".lo.$1").to_string(), Category::Trash)
            };
            dbg!(directory, &filename, &new_filename);
            match rename(
                format!("{directory}/{}", &filename),
                format!("{directory}/{}", &new_filename),
            ) {
                Ok(()) => {
                    model.set(
                        &iter,
                        &[
                            (Columns::Cat as u32, &new_cat.id()),
                            (Columns::Icon as u32, &new_cat.icon()),
                            (Columns::Name as u32, &new_filename),
                        ],
                    );
                    return true;
                }
                Err(e) => {
                    println!("Failed to rename {filename} to {new_filename}: {:?}", e)
                }
            }
        }
        false
    }

    fn set_sort_column(&self, sort_column_id: SortColumn, order: SortType) {
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        model.set_sort_column_id(sort_column_id, order);
    }

    fn set_unsorted(&self) {
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        model.set_unsorted();
    }
}
