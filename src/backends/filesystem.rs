use std::{
    ffi::OsStr,
    fs::{self, rename},
    io,
    path::Path,
    time::UNIX_EPOCH,
};

use eog::{Image, ImageData, ImageExt, Job};
use gio::File;
use glib::ObjectExt;
use gtk::{prelude::GtkListStoreExtManual, ListStore, TreeIter};

use regex::Regex;

use crate::{backends::TreeModelMviewExt, category::Category, draw::draw, filelistview::Direction};

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
                None
            }
        }
    }

    fn enter(&self, model: ListStore, iter: TreeIter) -> Box<dyn Backend> {
        Box::new(FileSystem::new(&format!(
            "{}/{}",
            self.directory,
            model.filename(&iter)
        )))
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

        let path = Path::new(&filename);
        let file = File::for_path(path);

        let cat = match fs::metadata(path) {
            Ok(metadata) => Category::determine(&filename, &metadata),
            Err(_) => Category::Unsupported,
        };

        let image = match cat {
            Category::Direcory | Category::Archive | Category::Unsupported => {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                draw(name)
            }
            _ => Ok(Image::new_file(&file, &filename)),
        };

        let image = image.unwrap();

        let filename_c = filename.clone();
        image.add_weak_ref_notify(move || {
            println!("**image [{filename_c}] disposed**");
        });

        match image.load(ImageData::IMAGE, None::<Job>.as_ref()) {
            Ok(()) => image,
            Err(error) => draw(&format!("Error {}", error)).unwrap(),
        }
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
