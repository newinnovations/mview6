mod imp;

use glib::{Cast, IsA};
use gtk::{
    glib,
    prelude::{TreeModelExt, TreeSortableExtManual, TreeViewExt},
    ListStore, TreeIter, TreePath, TreeView, TreeViewColumn,
};

use crate::{backends::TreeModelMviewExt, category::Category};

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
    fn navigate(&self, direction: Direction, filter: Filter, count: i32) -> bool;
    // fn set_sort_column(&self, sort_column_id: SortColumn, order: SortType);
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

    fn goto(&self, filename: &str) -> bool {
        println!("Goto {filename}");
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(iter) = model.iter_first() {
            loop {
                if filename == model.filename(&iter) {
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

                let cat = model.category(&iter);

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

    // fn set_sort_column(&self, sort_column_id: SortColumn, order: SortType) {
    //     let model = self.model().unwrap().downcast::<ListStore>().unwrap();
    //     model.set_sort_column_id(sort_column_id, order);
    // }

    fn set_unsorted(&self) {
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        model.set_unsorted();
    }
}
