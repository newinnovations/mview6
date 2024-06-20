use gtk::{ListStore, TreeIter};

use crate::filelistview::Direction;

use super::Backend;

pub struct Invalid {
    directory: String,
}

impl Invalid {
    pub fn new(directory: &str) -> Self {
        Invalid {
            directory: directory.to_string(),
        }
    }
}

impl Backend for Invalid {
    fn class_name(&self) -> &str {
        "Invalid"
    }

    fn directory(&self) -> &str {
        return "none";
    }

    fn create_store(&self) -> Option<ListStore> {
        println!("create_store Invalid {}", self.directory);
        None
    }

    fn favorite(&self, _model: ListStore, _iter: TreeIter, _direction: Direction) -> bool {
        false
    }
}
