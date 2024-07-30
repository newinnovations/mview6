use super::Image;
use crate::{
    category::Category,
    config::config,
    filelistview::{Columns, Cursor, Sort},
    image::draw::draw,
    window::MViewWidgets,
};
use gtk::{prelude::GtkListStoreExtManual, ListStore};
use std::{
    cell::{Cell, RefCell},
    fs, io,
    time::UNIX_EPOCH,
};

use super::{Backend, Selection};

pub struct Bookmarks {
    store: ListStore,
    parent: RefCell<Box<dyn Backend>>,
    sort: Cell<Sort>,
}

impl Bookmarks {
    pub fn new() -> Self {
        Bookmarks {
            store: Self::create_store(),
            parent: RefCell::new(<dyn Backend>::none()),
            sort: Default::default(),
        }
    }

    fn read_directory(store: &ListStore) -> io::Result<()> {
        let config = config();
        for entry in &config.bookmarks {
            let metadata = match fs::metadata(&entry.folder) {
                Ok(m) => m,
                Err(e) => {
                    println!("{}: Err = {:?}", &entry.folder, e);
                    continue;
                }
            };
            let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
            let modified = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
            let file_size = metadata.len();
            let cat = Category::Direcory;
            store.insert_with_values(
                None,
                &[
                    (Columns::Cat as u32, &cat.id()),
                    (Columns::Icon as u32, &cat.icon()),
                    (Columns::Name as u32, &entry.name),
                    (Columns::Folder as u32, &entry.folder),
                    (Columns::Size as u32, &file_size),
                    (Columns::Modified as u32, &modified),
                ],
            );
        }
        Ok(())
    }

    fn create_store() -> ListStore {
        let store = Columns::store();
        match Self::read_directory(&store) {
            Ok(()) => (),
            Err(e) => {
                println!("read_dir failed {:?}", e);
            }
        }
        store
    }
}

impl Backend for Bookmarks {
    fn class_name(&self) -> &str {
        "Bookmarks"
    }

    fn is_bookmarks(&self) -> bool {
        true
    }

    fn path(&self) -> &str {
        "/bookmarks"
    }

    fn store(&self) -> ListStore {
        self.store.clone()
    }

    fn enter(&self, cursor: &Cursor) -> Option<Box<dyn Backend>> {
        Some(<dyn Backend>::new(&cursor.folder()))
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        (self.parent.replace(<dyn Backend>::none()), Selection::None)
    }

    fn image(&self, _w: &MViewWidgets, cursor: &Cursor) -> Image {
        draw(&cursor.folder()).unwrap()
    }

    fn set_parent(&self, parent: Box<dyn Backend>) {
        self.parent.replace(parent);
    }

    fn set_sort(&self, sort: &Sort) {
        self.sort.set(*sort)
    }

    fn sort(&self) -> Sort {
        self.sort.get()
    }
}
