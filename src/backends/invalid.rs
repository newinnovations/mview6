use eog::Image;
use gtk::ListStore;

use crate::{draw::draw, filelistview::Cursor, window::MViewWidgets};

use super::{empty_store, Backend, Backends, Selection};

#[derive(Clone)]
pub struct Invalid {}

impl Invalid {
    pub fn new() -> Self {
        Invalid {}
    }
}

impl Default for Invalid {
    fn default() -> Self {
        Self::new()
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

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        (Box::new(Invalid::new()), Selection::None)
    }

    fn image(&self, _w: &MViewWidgets, _cursor: &Cursor) -> Image {
        draw("invalid").unwrap()
    }

    fn backend(&self) -> Backends {
        Backends::Invalid(self.clone())
    }
}
