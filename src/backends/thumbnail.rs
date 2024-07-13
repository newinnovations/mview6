use std::cell::RefCell;

use eog::Image;
use gtk::{
    prelude::{GtkListStoreExtManual, TreeModelExt},
    ListStore, TreeIter,
};

use crate::{category::Category, draw::draw};

use super::{empty_store, Backend, Columns, TreeModelMviewExt};

pub struct Thumbnail {
    parent: RefCell<Box<dyn Backend>>,
}

impl Thumbnail {
    pub fn new() -> Self {
        Thumbnail {
            parent: RefCell::new(<dyn Backend>::invalid()),
        }
    }
}

impl Backend for Thumbnail {
    fn class_name(&self) -> &str {
        "Thumbnail"
    }

    fn path(&self) -> &str {
        "/thumbnail"
    }

    fn store(&self) -> ListStore {
        let store = empty_store();
        // let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        // let modified = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
        // let file_size = metadata.len();
        let cat = Category::Direcory;
        let name = "Thumbnail page 1";
        store.insert_with_values(
            None,
            &[
                (Columns::Cat as u32, &cat.id()),
                (Columns::Icon as u32, &cat.icon()),
                (Columns::Name as u32, &name),
                // (Columns::Folder as u32, &entry.folder),
                // (Columns::Size as u32, &file_size),
                // (Columns::Modified as u32, &modified),
            ],
        );
        store
    }

    fn enter(&self, _model: ListStore, _iter: TreeIter) -> Box<dyn Backend> {
        Box::new(Thumbnail::new())
    }

    fn leave(&self) -> (Box<dyn Backend>, String) {
        (Box::new(Thumbnail::new()), "/".to_string())
    }

    fn image(&self, _model: ListStore, _iter: TreeIter) -> Image {

        let store = self.parent.borrow().store();

        let num_items = store.iter_n_children(None);
        dbg!(num_items);

        if let Some(i) = store.iter_nth_child(None, 0) {
            loop {
                let f = store.filename(&i);
                dbg!(&f);
                if !store.iter_next(&i) {
                    break;
                }
            }
        }
        draw("Thumbnail").unwrap()
    }

    fn set_parent(&self, parent: Box<dyn Backend>) {
        self.parent.replace(parent);
    }
}
