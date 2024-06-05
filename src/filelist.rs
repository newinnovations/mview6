use std::ffi::OsStr;
use std::fs;
use std::io;
use std::time::UNIX_EPOCH;

use gtk::prelude::GtkListStoreExtManual;

use crate::category::Category;

#[derive(Debug)]
#[repr(i32)]
pub enum Columns {
    Cat = 0,
    Icon,
    Name,
    Size,
    Modified,
}

pub struct FileList {
    directory: String,
}

fn read_directory(store: &gtk::ListStore, current_dir: &str) -> io::Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        let filename = path.file_name().unwrap_or(OsStr::new("-"));
        let filename = filename.to_str().unwrap_or("-");
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        let modified = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let file_size = metadata.len();

        let cat = Category::determine(filename, &metadata);

        store.insert_with_values(
            None,
            &[
                (Columns::Cat as u32, &cat.id()),
                (Columns::Icon as u32, &cat.icon()),
                (Columns::Name as u32, &filename),
                (Columns::Size as u32, &file_size),
                (Columns::Modified as u32, &modified),
            ],
        );
    }
    Ok(())
}

impl FileList {
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
        }
    }

    fn empty_model() -> gtk::ListStore {
        let col_types: [glib::Type; 5] = [
            glib::Type::U32,
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::U64,
            glib::Type::U64,
        ];
        gtk::ListStore::new(&col_types)
    }

    pub fn read(&self) -> gtk::ListStore {
        let store = Self::empty_model();
        let _ = read_directory(&store, &self.directory);
        store
    }

    pub fn read_new(&mut self, directory: &str) -> gtk::ListStore {
        self.directory = directory.to_string();
        self.read()
    }
}
