use eog::Image;
use gtk::{ListStore, TreeIter};

use crate::{draw::draw, window::MViewWidgets};

use super::{empty_store, Backend};

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

    fn path(&self) -> &str {
        "/invalid"
    }

    fn store(&self) -> ListStore {
        empty_store()
    }

    fn enter(&self, _model: ListStore, _iter: TreeIter) -> Box<dyn Backend> {
        Box::new(Invalid::new())
    }

    fn leave(&self) -> (Box<dyn Backend>, String) {
        (Box::new(Invalid::new()), "/".to_string())
    }

    fn image(&self, _w: &MViewWidgets, _model: &ListStore, _iter: &TreeIter) -> Image {
        draw("invalid").unwrap()
    }
}
