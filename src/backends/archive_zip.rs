use std::{
    fs,
    io::{BufReader, Read},
    path::Path,
};

use eog::Image;
use gtk::{prelude::GtkListStoreExtManual, ListStore, TreeIter};
use zip::result::ZipResult;

use crate::{
    backends::empty_store, category::Category, draw::draw, filelistview::Direction, loader::Loader,
};

use super::{filesystem::FileSystem, Backend, Columns, TreeModelMviewExt};

pub struct ZipArchive {
    filename: String,
}

impl ZipArchive {
    pub fn new(filename: &str) -> Self {
        ZipArchive {
            filename: filename.to_string(),
        }
    }
}

impl Backend for ZipArchive {
    fn class_name(&self) -> &str {
        "ZipArchive"
    }

    fn create_store(&self) -> Option<ListStore> {
        println!("create_store ZipArchive {}", self.filename);
        let store = empty_store();
        match list_zip(&self.filename, &store) {
            Ok(()) => println!("OK"),
            Err(e) => println!("ERROR {:?}", e),
        };
        Some(store)
    }

    fn favorite(&self, _model: ListStore, _iter: TreeIter, _direction: Direction) -> bool {
        false
    }

    fn enter(&self, _model: ListStore, _iter: TreeIter) -> Box<dyn Backend> {
        Box::new(ZipArchive::new(&self.filename))
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
        match extract_zip(&self.filename, model.index(&iter).try_into().unwrap()) {
            Ok(bytes) => Loader::image_from_memory(bytes),
            Err(error) => draw(&format!("Error {}", error)).unwrap(),
        }
    }
}

fn extract_zip(filename: &str, index: usize) -> ZipResult<Vec<u8>> {
    let fname = std::path::Path::new(filename);
    let file = fs::File::open(fname)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut file = archive.by_index(index)?;
    let mut buf = Vec::<u8>::new();
    let size = file.read_to_end(&mut buf)?;
    println!("extract_zip_to_memory::size={}", size);
    Ok(buf)
}

fn list_zip(filename: &str, store: &ListStore) -> ZipResult<()> {
    let fname = std::path::Path::new(filename);
    let file = fs::File::open(fname)?;
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;

        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => {
                println!("Entry {} has a suspicious path", file.name());
                continue;
            }
        };

        // {
        //     let comment = file.comment();
        //     if !comment.is_empty() {
        //         println!("Entry {i} comment: {comment}");
        //     }
        // }

        let filename = outpath.display().to_string();
        let cat = Category::determine(&filename, file.is_dir());
        let file_size = file.size();
        let modified = 0_u64;
        let index = i as u32;
        // let x=file.last_modified().unwrap();
        // x.q
        // if x.is_some()
        // .unwrap_or(UNIX_EPOCH);

        // if file.is_dir() {
        //     println!(
        //         "Entry {} is a directory with name \"{}\"",
        //         i,
        //         outpath.display()
        //     );
        // } else {
        //     println!(
        //         "Entry {} is a file with name \"{}\" ({} bytes)",
        //         i,
        //         outpath.display(),
        //         file.size()
        //     );
        // }

        store.insert_with_values(
            None,
            &[
                (Columns::Cat as u32, &cat.id()),
                (Columns::Icon as u32, &cat.icon()),
                (Columns::Name as u32, &filename),
                (Columns::Size as u32, &file_size),
                (Columns::Modified as u32, &modified),
                (Columns::Index as u32, &index),
            ],
        );
    }
    Ok(())
}
