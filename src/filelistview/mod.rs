mod cursor;
mod imp;
pub mod model;
mod sort;

pub use cursor::{Cursor, TreeModelMviewExt};
use glib::{Cast, IsA};
use gtk::{
    glib,
    prelude::{TreeModelExt, TreeSortableExtManual, TreeViewExt},
    ListStore, TreeView, TreeViewColumn,
};
pub use model::{Columns, Direction, Filter, Selection};
pub use sort::Sort;

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

pub trait FileListViewExt: IsA<FileListView> + IsA<TreeView> + 'static {
    fn goto(&self, selection: &Selection) -> bool;
    fn current(&self) -> Option<Cursor>;
    fn navigate(&self, direction: Direction, filter: Filter, count: i32);
    fn set_unsorted(&self);
}

impl<O: IsA<FileListView> + IsA<TreeView>> FileListViewExt for O {
    fn current(&self) -> Option<Cursor> {
        let (tree_path, _) = self.cursor();
        let store = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(path) = tree_path {
            store.iter(&path).map(|iter| Cursor {
                store,
                iter,
                position: *path.indices().first().unwrap_or(&0),
            })
        } else {
            store.iter_first().map(|iter| Cursor {
                store,
                iter,
                position: 0,
            })
        }
    }

    fn goto(&self, selection: &Selection) -> bool {
        // println!("Goto {:?}", selection);
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(iter) = model.iter_first() {
            loop {
                let found = match selection {
                    Selection::Name(filename) => *filename == model.name(&iter),
                    Selection::Index(index) => *index == model.index(&iter),
                    Selection::None => true,
                };
                if found {
                    let tp = model.path(&iter).unwrap_or_default();
                    self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                    return true;
                }
                if !model.iter_next(&iter) {
                    return false;
                }
            }
        } else {
            false
        }
    }

    fn navigate(&self, direction: Direction, filter: Filter, count: i32) {
        if let Some(current) = self.current() {
            if let Some(tree_path) = current.navigate(direction, filter, count) {
                self.set_cursor(&tree_path, None::<&TreeViewColumn>, false);
            }
        }
    }

    fn set_unsorted(&self) {
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        model.set_unsorted();
    }
}
