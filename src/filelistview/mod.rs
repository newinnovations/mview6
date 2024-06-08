mod imp;

use glib::{Cast, IsA};
use gtk::{
    glib,
    prelude::{GtkListStoreExtManual, TreeModelExt, TreeViewExt},
    ListStore, TreeIter, TreePath, TreeView, TreeViewColumn,
};

use crate::{category::Category, filelist::Columns};

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

pub trait FileListViewExt: IsA<FileListView> + IsA<TreeView> + 'static {
    fn goto_first(&self);
    fn goto(&self, filename: &str) -> bool;
    fn iter(&self) -> Option<(ListStore, TreeIter)>;
    fn current_filename(&self) -> Option<String>;
    fn write(&self);
    fn navigate(&self, direction: Direction, filter: Filter, count: i32) -> bool;
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
            None
        }
    }

    fn current_filename(&self) -> Option<String> {
        if let Some((model, iter)) = self.iter() {
            let filename = model
                .value(&iter, Columns::Name as i32)
                .get::<String>()
                .unwrap_or_default();
            Some(filename)
        } else {
            None
        }
    }

    fn write(&self) {
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        let iter = model.iter_first().unwrap();
        // model.set_value(&iter, Columns::Name as u32, &Value::from("xxx"));
        let c = 100_u32;
        model.set(
            &iter,
            &[(Columns::Cat as u32, &c), (Columns::Name as u32, &"blah")],
        )
    }

    fn goto(&self, filename: &str) -> bool {
        println!("Goto {filename}");
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(iter) = model.iter_first() {
            loop {
                let entry = model
                    .value(&iter, Columns::Name as i32)
                    .get::<String>()
                    .unwrap_or("none".to_string());
                if entry == filename {
                    let tp = model.path(&iter).unwrap_or_default();
                    self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                    return true;
                }
                if !model.iter_next(&iter) {
                    return false;
                }
            }
        }
        false
    }

    fn navigate(&self, direction: Direction, filter: Filter, count: i32) -> bool {
        if let Some((model, iter)) = self.iter() {
            let mut cnt = count;
            loop {
                let last = iter.clone();
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

                let cat = model
                    .value(&iter, Columns::Cat as i32)
                    .get::<u32>()
                    .unwrap_or(Category::Unsupported.id());

                let skip = match filter {
                    Filter::None => false,
                    Filter::Image => cat != Category::Image.id() && cat != Category::Favorite.id(),
                    Filter::Favorite => cat != Category::Favorite.id(),
                };

                if skip {
                    continue;
                }

                cnt = cnt - 1;
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
}
