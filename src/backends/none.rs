use eog::Image;
use gtk::ListStore;

use crate::{
    draw::draw,
    filelistview::{Columns, Cursor, Sort},
    window::MViewWidgets,
};

use super::{Backend, Selection};

#[derive(Clone)]
pub struct NoneBackend {}

impl NoneBackend {
    pub fn new() -> Self {
        NoneBackend {}
    }
}

impl Default for NoneBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for NoneBackend {
    fn class_name(&self) -> &str {
        "Invalid"
    }

    fn is_none(&self) -> bool {
        true
    }

    fn path(&self) -> &str {
        "/invalid"
    }

    fn store(&self) -> ListStore {
        Columns::store()
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        (Box::new(NoneBackend::new()), Selection::None)
    }

    fn image(&self, _w: &MViewWidgets, _cursor: &Cursor) -> Image {
        draw("invalid").unwrap()
    }

    fn set_sort(&self, _sort: &Sort) {}

    fn sort(&self) -> Sort {
        Default::default()
    }
}
