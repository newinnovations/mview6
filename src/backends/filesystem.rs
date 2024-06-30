use crate::{
    backends::TreeModelMviewExt, category::Category, filelistview::Direction, loader::Loader,
};
use eog::Image;
use gtk::{prelude::GtkListStoreExtManual, ListStore, TreeIter};
use regex::Regex;
use std::{
    ffi::OsStr,
    fs::{self, rename},
    io,
    path::Path,
    time::UNIX_EPOCH,
};

use super::{empty_store, Backend, Columns};

pub struct FileSystem {
    directory: String,
}

impl FileSystem {
    pub fn new(directory: &str) -> Self {
        FileSystem {
            directory: directory.to_string(),
        }
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

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    println!("{}: Err = {:?}", filename, e);
                    continue;
                }
            };

            let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
            let modified = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
            let file_size = metadata.len();

            let cat = Category::determine(filename, metadata.is_dir());

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
}

impl Backend for FileSystem {
    fn class_name(&self) -> &str {
        "FileSystem"
    }

    fn create_store(&self) -> Option<ListStore> {
        let store = empty_store();
        match Self::read_directory(&store, &self.directory) {
            Ok(()) => Some(store),
            Err(e) => {
                println!("read_dir failed {:?}", e);
                // None
                Some(store)
            }
        }
    }

    fn enter(&self, model: ListStore, iter: TreeIter) -> Box<dyn Backend> {
        <dyn Backend>::new(&format!("{}/{}", self.directory, model.filename(&iter)))
    }

    fn leave(&self) -> (Box<dyn Backend>, String) {
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
            Some(parent) => (
                Box::new(FileSystem::new(parent.to_str().unwrap_or("/"))),
                current,
            ),
            _ => (Box::new(FileSystem::new("/")), current),
        }
    }

    fn image(&self, model: ListStore, iter: TreeIter) -> Image {
        let filename = format!("{}/{}", self.directory, model.filename(&iter));
        Loader::image_from_file(&filename)
    }

    fn favorite(&self, model: ListStore, iter: TreeIter, direction: Direction) -> bool {
        let cat = model.category(&iter);
        if cat != Category::Image.id()
            && cat != Category::Favorite.id()
            && cat != Category::Trash.id()
        {
            return false;
        }

        let filename = model.filename(&iter);
        let re = Regex::new(r"\.([^\.]+)$").unwrap();
        let (new_filename, new_cat) = if matches!(direction, Direction::Up) {
            if filename.contains(".hi.") {
                return false;
            } else if filename.contains(".lo.") {
                (filename.replace(".lo", ""), Category::Image)
            } else {
                (
                    re.replace(&filename, ".hi.$1").to_string(),
                    Category::Favorite,
                )
            }
        } else if filename.contains(".lo.") {
            return false;
        } else if filename.contains(".hi.") {
            (filename.replace(".hi", ""), Category::Image)
        } else {
            (re.replace(&filename, ".lo.$1").to_string(), Category::Trash)
        };
        dbg!(&self.directory, &filename, &new_filename);
        match rename(
            format!("{}/{}", self.directory, &filename),
            format!("{}/{}", self.directory, &new_filename),
        ) {
            Ok(()) => {
                model.set(
                    &iter,
                    &[
                        (Columns::Cat as u32, &new_cat.id()),
                        (Columns::Icon as u32, &new_cat.icon()),
                        (Columns::Name as u32, &new_filename),
                    ],
                );
                true
            }
            Err(e) => {
                println!("Failed to rename {filename} to {new_filename}: {:?}", e);
                false
            }
        }
    }
}
