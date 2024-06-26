use std::path::Path;

use eog::Image;
use gtk::{prelude::GtkListStoreExtManual, ListStore, TreeIter};
use unrar::{error::UnrarError, Archive, UnrarResult};

use crate::{
    backends::empty_store, category::Category, draw::draw, filelistview::Direction, loader::Loader,
};

use super::{filesystem::FileSystem, Backend, Columns, TreeModelMviewExt};

pub struct RarArchive {
    filename: String,
}

impl RarArchive {
    pub fn new(filename: &str) -> Self {
        RarArchive {
            filename: filename.to_string(),
        }
    }
}

impl Backend for RarArchive {
    fn class_name(&self) -> &str {
        "RarArchive"
    }

    fn create_store(&self) -> Option<ListStore> {
        println!("create_store RarArchive {}", &self.filename);
        let store = empty_store();
        match list_rar(&self.filename, &store) {
            Ok(()) => println!("OK"),
            Err(e) => println!("ERROR {:?}", e),
        };
        Some(store)
    }

    fn favorite(&self, _model: ListStore, _iter: TreeIter, _direction: Direction) -> bool {
        false
    }

    fn enter(&self, _model: ListStore, _iter: TreeIter) -> Box<dyn Backend> {
        Box::new(RarArchive::new(&self.filename))
    }

    fn leave(&self) -> (Box<dyn Backend>, String) {
        let path = Path::new(&self.filename);
        let directory = path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_str()
            .unwrap_or("/");
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        (Box::new(FileSystem::new(directory)), filename.to_string())
    }

    fn image(&self, model: ListStore, iter: TreeIter) -> Image {
        let sel = model.filename(&iter);
        match extract_rar(&self.filename, &sel) {
            Ok(bytes) => Loader::image_from_memory(bytes),
            Err(error) => draw(&format!("Error {}", error)).unwrap(),
        }
    }
}

fn extract_rar(filename: &str, sel: &str) -> UnrarResult<Vec<u8>> {
    let mut archive = Archive::new(filename).open_for_processing()?;
    while let Some(header) = archive.read_header()? {
        let e_filename = header.entry().filename.as_os_str().to_str().unwrap_or("-");
        archive = if header.entry().is_file() {
            if e_filename == sel {
                let (bytes, _) = header.read()?;
                return Ok(bytes);
            } else {
                header.skip()?
            }
        } else {
            header.skip()?
        };
    }
    Err(UnrarError {
        code: unrar::error::Code::EndArchive,
        when: unrar::error::When::Read,
    })
}

fn list_rar(filename: &str, store: &ListStore) -> UnrarResult<()> {
    let archive = Archive::new(&filename).open_for_listing()?;
    for e in archive {
        let entry = e?;
        let filename = entry.filename.as_os_str().to_str().unwrap_or("???");
        let cat = Category::determine(filename, false); //file.is_dir());
        let file_size = entry.unpacked_size; // file.size();
        let modified = 0_u64;
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
