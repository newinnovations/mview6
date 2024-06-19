use gtk::ListStore;

use super::Backend;

pub struct FileSystem {
    directory: String,
}

impl FileSystem {
    pub fn new(directory: &str) -> Self {
        FileSystem {
            directory: directory.to_string(),
        }
    }
}

impl Backend for FileSystem {
    fn class_name(&self) -> &str {
        "FileSystem"
    }

    fn create_store(&self) -> Option<ListStore> {
        println!("create_store FileSystem {}", self.directory);
        None
    }
}
