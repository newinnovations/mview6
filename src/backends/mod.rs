use std::env;

use archive_rar::RarArchive;
use archive_zip::ZipArchive;
use bookmarks::Bookmarks;
use filesystem::FileSystem;
use gtk4::ListStore;
use none::NoneBackend;
use thumbnail::{TEntry, Thumbnail};

use crate::{
    filelistview::{Cursor, Direction, Selection, Sort},
    image::Image,
    window::MViewWidgets,
};

mod archive_rar;
mod archive_zip;
mod bookmarks;
pub mod filesystem;
mod none;
pub mod thumbnail;

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
    fn entry(&self, cursor: &Cursor) -> TEntry {
        Default::default()
    }
    fn is_container(&self) -> bool {
        false
    }
    fn is_bookmarks(&self) -> bool {
        false
    }
    fn is_thumbnail(&self) -> bool {
        false
    }
    fn is_none(&self) -> bool {
        false
    }
    fn click(&self, current: &Cursor, x: f64, y: f64) -> Option<(Box<dyn Backend>, Selection)> {
        None
    }
    fn set_parent(&self, parent: Box<dyn Backend>) {}
    fn set_sort(&self, sort: &Sort);
    fn sort(&self) -> Sort;
}

impl std::fmt::Debug for dyn Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Backend({})", self.class_name())
    }
}

impl Default for Box<dyn Backend> {
    fn default() -> Self {
        Box::new(NoneBackend::new())
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

    pub fn none() -> Box<dyn Backend> {
        Box::new(NoneBackend::new())
    }

    pub fn current_dir() -> Box<dyn Backend> {
        match env::current_dir() {
            Ok(cwd) => Box::new(FileSystem::new(cwd.as_os_str().to_str().unwrap_or("/"))),
            Err(_) => Box::new(FileSystem::new("/")),
        }
    }
}
