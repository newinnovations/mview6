use std::env;

use archive_rar::RarArchive;
use archive_zip::ZipArchive;
use bookmarks::Bookmarks;
use eog::Image;
use filesystem::FileSystem;
use glib::IsA;
use gtk::{
    prelude::{TreeModelExt, TreeSortableExtManual},
    ListStore, TreeIter, TreeModel,
};
use invalid::Invalid;
use thumbnail::{TEntry, Thumbnail};

use crate::{
    category::Category,
    filelistview::{Cursor, Direction},
    window::MViewWidgets,
};

mod archive_rar;
mod archive_zip;
mod bookmarks;
pub mod filesystem;
mod invalid;
pub mod thumbnail;

#[derive(Debug)]
#[repr(u32)]
pub enum Columns {
    Cat = 0,
    Icon,
    Name,
    Size,
    Modified,
    Index,
    Folder,
}

#[derive(Debug)]
pub enum Selection {
    Name(String),
    Index(u32),
    None,
}

impl From<TEntry> for Selection {
    fn from(item: TEntry) -> Self {
        match item {
            TEntry::FileEntry(file) => Selection::Name(file.filename()),
            TEntry::ZipEntry(zip) => Selection::Index(zip.index()),
            TEntry::RarEntry(rar) => Selection::Name(rar.selection()),
            TEntry::None => Selection::None,
        }
    }
}

#[allow(unused_variables)]
pub trait Backend {
    fn class_name(&self) -> &str;
    fn path(&self) -> &str;
    fn store(&self) -> ListStore;
    fn favorite(&self, cursor: &Cursor, direction: Direction) -> bool {
        false
    }
    fn enter(&self, cursor: &Cursor) -> Option<Box<dyn Backend>> {
        None
    }
    fn leave(&self) -> (Box<dyn Backend>, Selection);
    fn image(&self, w: &MViewWidgets, cursor: &Cursor) -> Image;
    fn entry(&self, model: &ListStore, iter: &TreeIter) -> TEntry {
        TEntry::None
    }
    fn set_parent(&self, parent: Box<dyn Backend>) {}
    fn backend(&self) -> Backends;
    fn is_thumbnail(&self) -> bool {
        false
    }
    fn click(&self, current: &Cursor, x: f64, y: f64) -> Option<(Box<dyn Backend>, Selection)> {
        None
    }
}

impl std::fmt::Debug for dyn Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Backend({})", self.class_name())
    }
}

impl dyn Backend {
    pub fn new(filename: &str) -> Box<dyn Backend> {
        if filename.ends_with(".zip") {
            Box::new(ZipArchive::new(filename))
        } else if filename.ends_with(".rar") {
            Box::new(RarArchive::new(filename))
        } else {
            Box::new(FileSystem::new(filename))
        }
    }

    pub fn bookmarks() -> Box<dyn Backend> {
        Box::new(Bookmarks::new())
    }

    pub fn thumbnail(thumbnail: Thumbnail) -> Box<dyn Backend> {
        Box::new(thumbnail)
    }

    pub fn invalid() -> Box<dyn Backend> {
        Box::new(Invalid::new())
    }

    pub fn current_dir() -> Box<dyn Backend> {
        match env::current_dir() {
            Ok(cwd) => Box::new(FileSystem::new(cwd.as_os_str().to_str().unwrap_or("/"))),
            Err(_) => Self::invalid(),
        }
    }
}

pub enum Backends {
    File(FileSystem),
    Zip(ZipArchive),
    Rar(RarArchive),
    Invalid(Invalid),
}

impl Backends {
    fn dynbox(&self) -> Box<dyn Backend> {
        match self {
            Backends::File(f) => Box::new(f.clone()),
            Backends::Zip(f) => Box::new(f.clone()),
            Backends::Rar(f) => Box::new(f.clone()),
            Backends::Invalid(f) => Box::new(f.clone()),
        }
    }
}

pub trait TreeModelMviewExt: IsA<TreeModel> {
    fn name(&self, iter: &TreeIter) -> String;
    fn folder(&self, iter: &TreeIter) -> String;
    fn category(&self, iter: &TreeIter) -> u32;
    fn index(&self, iter: &TreeIter) -> u32;
}

impl<O: IsA<TreeModel>> TreeModelMviewExt for O {
    fn name(&self, iter: &TreeIter) -> String {
        self.value(iter, Columns::Name as i32)
            .get::<String>()
            .unwrap_or_default()
    }
    fn folder(&self, iter: &TreeIter) -> String {
        self.value(iter, Columns::Folder as i32)
            .get::<String>()
            .unwrap_or_default()
    }
    fn category(&self, iter: &TreeIter) -> u32 {
        self.value(iter, Columns::Cat as i32)
            .get::<u32>()
            .unwrap_or(Category::Unsupported.id())
    }
    fn index(&self, iter: &TreeIter) -> u32 {
        self.value(iter, Columns::Index as i32)
            .get::<u32>()
            .unwrap_or(0)
    }
}

pub fn empty_store() -> ListStore {
    let col_types: [glib::Type; 7] = [
        glib::Type::U32,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::U64,
        glib::Type::U64,
        glib::Type::U32,
        glib::Type::STRING,
    ];
    let store = ListStore::new(&col_types);
    store.set_sort_func(
        gtk::SortColumn::Index(Columns::Cat as u32),
        |model, iter1, iter2| {
            let cat1 = model.category(iter1);
            let cat2 = model.category(iter2);
            let result = cat1.cmp(&cat2);
            if result.is_eq() {
                let filename1 = model.name(iter1).to_lowercase();
                let filename2 = model.name(iter2).to_lowercase();
                filename1.cmp(&filename2)
            } else {
                result
            }
        },
    );
    store
}
