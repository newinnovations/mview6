use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::time::UNIX_EPOCH;

use gtk::prelude::GtkListStoreExtManual;
use gtk::prelude::TreeModelExt;
use gtk::prelude::TreeSortableExtManual;
use gtk::ListStore;
use gtk::TreeIter;
use gtk::TreeModel;

use crate::backends::Columns;
use crate::category::Category;

#[derive(Debug)]
pub struct FileList {
    pub directory: String,
}

fn read_directory(store: &ListStore, current_dir: &str) -> io::Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name().unwrap_or(OsStr::new("-"));
        let filename = filename.to_str().unwrap_or("-");

        if filename.starts_with('.') {
            continue;
        }

        let metadata = fs::metadata(&path)?;
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

// TODO: move to trait or new store type
fn model_filename(model: &TreeModel, iter: &TreeIter) -> String {
    model
        .value(iter, Columns::Name as i32)
        .get::<String>()
        .unwrap_or_default()
}

fn model_category(model: &TreeModel, iter: &TreeIter) -> u32 {
    model
        .value(iter, Columns::Cat as i32)
        .get::<u32>()
        .unwrap_or(Category::Unsupported.id())
}

impl FileList {
    // extern nodig in initiele lijst (window construct)
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
        }
    }

    // extern nodig in initiele lijst (window construct)
    pub fn read(&self) -> Option<ListStore> {
        Self::create_store(&self.directory)
    }

    // extern nodig voor window::navigate_to(file)
    pub fn goto(&mut self, directory: &str) -> Option<ListStore> {
        let newstore = Self::create_store(directory);
        if newstore.is_some() {
            self.directory = directory.to_string();
        }
        newstore
    }

    // extern nodig voor nodig voor favorite
    pub fn directory(&self) -> String {
        self.directory.clone()
    }

    fn empty_store() -> ListStore {
        let col_types: [glib::Type; 5] = [
            glib::Type::U32,
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::U64,
            glib::Type::U64,
        ];
        let store = ListStore::new(&col_types);
        store.set_sort_func(
            gtk::SortColumn::Index(Columns::Cat as u32),
            |model, iter1, iter2| {
                let cat1 = model_category(model, iter1);
                let cat2 = model_category(model, iter2);
                let result = cat1.cmp(&cat2);
                if result.is_eq() {
                    let filename1 = model_filename(model, iter1).to_lowercase();
                    let filename2 = model_filename(model, iter2).to_lowercase();
                    filename1.cmp(&filename2)
                } else {
                    result
                }
            },
        );
        store
    }

    fn create_store(directory: &str) -> Option<ListStore> {
        let store = Self::empty_store();
        match read_directory(&store, directory) {
            Ok(()) => Some(store),
            Err(e) => {
                println!("read_dir failed {:?}", e);
                None
            }
        }
    }

    pub fn enter(&mut self, subdir: &str) -> Option<ListStore> {
        self.goto(&format!("{0}/{subdir}", self.directory))
    }

    pub fn leave(&mut self) -> Option<(ListStore, String)> {
        let directory_c = self.directory.clone();
        let directory_p = Path::new(&directory_c);
        let parent = directory_p.parent();
        let current = directory_p
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        match parent {
            Some(parent) => self
                .goto(parent.to_str().unwrap_or("/"))
                .map(|model| (model, current)),
            _ => None,
        }
    }
}
