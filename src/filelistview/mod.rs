mod cursor;
mod imp;
pub mod model;
mod sort;

pub use cursor::{Cursor, TreeModelMviewExt};
use glib::object::{Cast, IsA};
use gtk4::{
    glib,
    prelude::{TreeModelExt, TreeSortableExtManual, TreeViewExt},
    ListStore, TreeView, TreeViewColumn,
};
pub use model::{Columns, Direction, Filter, Selection};
pub use sort::Sort;

glib::wrapper! {
pub struct FileListView(ObjectSubclass<imp::FileListViewImp>)
    @extends gtk4::Widget, gtk4::TreeView, gtk4::Scrollable;
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
    fn store(&self) -> Option<ListStore>;
    fn goto(&self, selection: &Selection) -> bool;
    fn current(&self) -> Option<Cursor>;
    fn home(&self);
    fn end(&self);
    fn navigate(&self, direction: Direction, filter: Filter, count: i32);
    fn set_unsorted(&self);
}

impl<O: IsA<FileListView> + IsA<TreeView>> FileListViewExt for O {
    fn store(&self) -> Option<ListStore> {
        if let Some(model) = self.model() {
            model.downcast::<ListStore>().ok()
        } else {
            None
        }
    }

    fn current(&self) -> Option<Cursor> {
        let (tree_path, _) = self.cursor();
        if let Some(store) = self.store() {
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
        } else {
            None
        }
    }

    fn goto(&self, selection: &Selection) -> bool {
        // println!("Goto {:?}", selection);
        if let Some(store) = self.store() {
            if let Some(iter) = store.iter_first() {
                loop {
                    let found = match selection {
                        Selection::Name(filename) => *filename == store.name(&iter),
                        Selection::Index(index) => *index == store.index(&iter),
                        Selection::None => true,
                    };
                    if found {
                        let tp = store.path(&iter); //.unwrap_or_default();
                        self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                        return true;
                    }
                    if !store.iter_next(&iter) {
                        return false;
                    }
                }
            }
        }
        false
    }

    fn home(&self) {
        if let Some(store) = self.store() {
            if let Some(iter) = store.iter_first() {
                let tp = store.path(&iter);
                self.set_cursor(&tp, None::<&TreeViewColumn>, false);
            }
        }
    }

    fn end(&self) {
        if let Some(store) = self.store() {
            let num_items = store.iter_n_children(None);
            if num_items > 1 {
                if let Some(iter) = store.iter_nth_child(None, num_items - 1) {
                    let tp = store.path(&iter);
                    self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                }
            }
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
        if let Some(store) = self.store() {
            store.set_unsorted();
        }
    }
}
