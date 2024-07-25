use crate::{
    category::Category,
    error::MviewResult,
    filelistview::{Cursor, Direction},
    image::{ImageLoader, ImageSaver},
    window::MViewWidgets,
};
use eog::Image;
use gtk::{prelude::GtkListStoreExtManual, ListStore};
use image::DynamicImage;
use regex::Regex;
use std::{
    ffi::OsStr,
    fs::{self, rename},
    io,
    path::Path,
    time::UNIX_EPOCH,
};

use super::{
    empty_store,
    thumbnail::{TEntry, TReference},
    Backend, Backends, Columns, Selection,
};

#[derive(Clone)]
pub struct FileSystem {
    directory: String,
    store: ListStore,
}

impl FileSystem {
    pub fn new(directory: &str) -> Self {
        FileSystem {
            directory: directory.to_string(),
            store: Self::create_store(directory),
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

    fn create_store(directory: &str) -> ListStore {
        let store = empty_store();
        match Self::read_directory(&store, directory) {
            Ok(()) => (),
            Err(e) => {
                println!("read_dir failed {:?}", e);
            }
        }
        store
    }

    pub fn get_thumbnail(src: &TFileReference) -> MviewResult<DynamicImage> {
        let thumb_filename = src.filename.replace(".lo.", ".").replace(".hi.", ".") + ".mthumb";
        let thumb_path = format!("{}/.mview/{}", src.directory, thumb_filename);
        if Path::new(&thumb_path).exists() {
            ImageLoader::dynimg_from_file(&thumb_path)
        } else {
            let path = format!("{}/{}", src.directory, src.filename);
            let image = ImageLoader::dynimg_from_file(&path)?;
            let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
            ImageSaver::save_thumbnail(&src.directory, &thumb_filename, &image);
            Ok(image)
        }
    }
}

impl Backend for FileSystem {
    fn class_name(&self) -> &str {
        "FileSystem"
    }

    fn path(&self) -> &str {
        &self.directory
    }

    fn store(&self) -> ListStore {
        self.store.clone()
    }

    fn enter(&self, cursor: &Cursor) -> Option<Box<dyn Backend>> {
        let category = cursor.category();
        if category == Category::Direcory.id() || category == Category::Archive.id() {
            Some(<dyn Backend>::new(&format!(
                "{}/{}",
                self.directory,
                cursor.name()
            )))
        } else {
            None
        }
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
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
                Selection::Name(current),
            ),
            _ => (Box::new(FileSystem::new("/")), Selection::Name(current)),
        }
    }

    fn image(&self, _w: &MViewWidgets, cursor: &Cursor) -> Image {
        let filename = format!("{}/{}", self.directory, cursor.name());
        ImageLoader::image_from_file(&filename)
    }

    fn favorite(&self, cursor: &Cursor, direction: Direction) -> bool {
        let cat = cursor.category();
        if cat != Category::Image.id()
            && cat != Category::Favorite.id()
            && cat != Category::Trash.id()
        {
            return false;
        }

        let filename = cursor.name();
        let re = Regex::new(r"\.([^\.]+)$").unwrap();
        let (new_filename, new_category) = if matches!(direction, Direction::Up) {
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
                cursor.update(new_category, &new_filename);
                true
            }
            Err(e) => {
                println!("Failed to rename {filename} to {new_filename}: {:?}", e);
                false
            }
        }
    }

    fn entry(&self, cursor: &Cursor) -> TEntry {
        let name = &cursor.name();
        TEntry::new(
            Category::from(cursor.category()),
            name,
            TReference::FileReference(TFileReference::new(&self.directory, name)),
        )
    }

    fn backend(&self) -> Backends {
        Backends::File(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct TFileReference {
    directory: String,
    filename: String,
}

impl TFileReference {
    pub fn new(directory: &str, filename: &str) -> Self {
        TFileReference {
            directory: directory.to_string(),
            filename: filename.to_string(),
        }
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }
}
