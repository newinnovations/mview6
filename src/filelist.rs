use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::time::UNIX_EPOCH;

use gtk::prelude::GtkListStoreExtManual;
use gtk::ListStore;

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
    pub directory: String,
}

fn read_directory(store: &ListStore, current_dir: &str) -> io::Result<()> {
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

    fn empty_store() -> ListStore {
        let col_types: [glib::Type; 5] = [
            glib::Type::U32,
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::U64,
            glib::Type::U64,
        ];
        ListStore::new(&col_types)
    }

    fn read_dir(directory: &str) -> Option<ListStore> {
        let store = Self::empty_store();
        match read_directory(&store, directory) {
            Ok(()) => Some(store),
            _ => None,
        }
    }

    pub fn read(&self) -> Option<ListStore> {
        Self::read_dir(&self.directory)
    }

    pub fn goto(&mut self, directory: &str) -> Option<ListStore> {
        let newstore = Self::read_dir(directory);
        match newstore {
            Some(_) => self.directory = directory.to_string(),
            _ => (),
        }
        newstore
    }

    pub fn enter(&mut self, subdir: &str) -> Option<ListStore> {
        self.goto(&format!("{0}/{subdir}", self.directory))
    }

    pub fn leave(&mut self) -> Option<ListStore> {
        let directory_c = self.directory.clone();
        let parent = Path::new(&directory_c).parent();
        match parent {
            Some(parent) => self.goto(parent.to_str().unwrap_or("/")),
            _ => None,
        }
    }
}
