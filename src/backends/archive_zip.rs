use std::{
    fs,
    io::{BufReader, Read},
    path::Path,
};

use chrono::{Local, TimeZone};
use eog::Image;
use gtk::{prelude::GtkListStoreExtManual, ListStore, TreeIter};
use zip::result::ZipResult;

use crate::{
    backends::empty_store, category::Category, draw::draw, loader::Loader, window::MViewWidgets,
};

use super::{filesystem::FileSystem, Backend, Columns, TreeModelMviewExt};

pub struct ZipArchive {
    filename: String,
    store: ListStore,
}

impl ZipArchive {
    pub fn new(filename: &str) -> Self {
        ZipArchive {
            filename: filename.to_string(),
            store: Self::create_store(filename),
        }
    }

    fn create_store(filename: &str) -> ListStore {
        println!("create_store ZipArchive {}", filename);
        let store = empty_store();
        match list_zip(filename, &store) {
            Ok(()) => println!("OK"),
            Err(e) => println!("ERROR {:?}", e),
        };
        store
    }
}

impl Backend for ZipArchive {
    fn class_name(&self) -> &str {
        "ZipArchive"
    }

    fn path(&self) -> &str {
        &self.filename
    }

    fn store(&self) -> ListStore {
        self.store.clone()
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

    fn image(&self, _w: &MViewWidgets, model: &ListStore, iter: &TreeIter) -> Image {
        match extract_zip(&self.filename, model.index(iter).try_into().unwrap()) {
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

        let filename = outpath.display().to_string();
        let cat = Category::determine(&filename, file.is_dir());
        let file_size = file.size();
        let index = i as u32;

        if file_size == 0 {
            continue;
        }

        if cat.id() == Category::Unsupported.id() {
            continue;
        }

        let m = file.last_modified().unwrap_or_default();
        let modified = match Local.with_ymd_and_hms(
            m.year() as i32,
            m.month() as u32,
            m.day() as u32,
            m.hour() as u32,
            m.minute() as u32,
            m.second() as u32,
        ) {
            chrono::offset::LocalResult::Single(datetime) => datetime.timestamp() as u64,
            _ => {
                println!("Could not create local datetime (Ambiguous or None)");
                0_u64
            }
        };

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
