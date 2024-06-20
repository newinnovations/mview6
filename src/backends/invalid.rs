use eog::Image;
use gtk::{ListStore, TreeIter};

use crate::{draw::draw, filelistview::Direction};

use super::Backend;

pub struct Invalid {}

impl Invalid {
    pub fn new() -> Self {
        Invalid {}
    }
}

impl Backend for Invalid {
    fn class_name(&self) -> &str {
        "Invalid"
    }

    fn create_store(&self) -> Option<ListStore> {
        println!("create_store Invalid");
        None
    }

    fn favorite(&self, _model: ListStore, _iter: TreeIter, _direction: Direction) -> bool {
        false
    }

    fn enter(&self, _model: ListStore, _iter: TreeIter) -> Box<dyn Backend> {
        Box::new(Invalid::new())
    }

    fn leave(&self) -> (Box<dyn Backend>, String) {
        (Box::new(Invalid::new()), "/".to_string())
    }

    fn image(&self, _model: ListStore, _iter: TreeIter) -> Image {
        draw("invalid").unwrap()
    }
}
