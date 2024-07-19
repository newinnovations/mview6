use std::{
    fs,
    io::{BufReader, Read},
    path::Path,
};

use chrono::{Local, TimeZone};
use eog::Image;
use gtk::{prelude::GtkListStoreExtManual, ListStore, TreeIter};
use image::DynamicImage;
use zip::result::ZipResult;

use crate::{
    backends::empty_store, category::Category, draw::draw, loader::Loader, window::MViewWidgets,
};

use super::{
    filesystem::FileSystem, thumbnail::TSource, Backend, Backends, Columns, Selection,
    TreeModelMviewExt,
};

#[derive(Clone)]
pub struct ZipArchive {
    filename: String,
    directory: String,
    archive: String,
    store: ListStore,
}

impl ZipArchive {
    pub fn new(filename: &str) -> Self {
        let path = Path::new(filename);
        let directory = path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_str()
            .unwrap_or("/");
        let archive = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        ZipArchive {
            filename: filename.to_string(),
            directory: directory.to_string(),
            archive: archive.to_string(),
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

    pub fn get_thumbnail(src: &TZipSource) -> Option<DynamicImage> {
        let thumb_filename = format!(
            "{}/.mview/{}-{}.mthumb",
            src.directory, src.archive, src.index
        );
        if Path::new(&thumb_filename).exists() {
            if let Ok(im) = Loader::dynimg_from_file(&thumb_filename) {
                Some(im)
            } else {
                None
            }
        } else {
            let img = match extract_zip(&src.filename, src.index as usize) {
                Ok(bytes) => Loader::dynimg_from_memory(&bytes),
                Err(_error) => return None,
            };
            if let Ok(im) = img {
                let im = im.resize(175, 175, image::imageops::FilterType::Lanczos3);
                let thumb_dir = format!("{}/.mview", src.directory);
                if !Path::new(&thumb_dir).exists() {
                    let _ = fs::create_dir(thumb_dir);
                }
                let _ = im.save_with_format(thumb_filename, image::ImageFormat::Jpeg);
                Some(im)
            } else {
                None
            }
        }
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

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        (
            Box::new(FileSystem::new(&self.directory)),
            Selection::Name(self.archive.clone()),
        )
    }

    fn image(&self, _w: &MViewWidgets, model: &ListStore, iter: &TreeIter) -> Image {
        match extract_zip(&self.filename, model.index(iter).try_into().unwrap()) {
            Ok(bytes) => Loader::image_from_memory(bytes),
            Err(error) => draw(&format!("Error {}", error)).unwrap(),
        }
    }

    fn thumb(&self, model: &ListStore, iter: &TreeIter) -> TSource {
        TSource::ZipSource(TZipSource::new(self, model.index(iter)))
    }

    fn backend(&self) -> Backends {
        Backends::Zip(self.clone())
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

#[derive(Debug, Clone)]
pub struct TZipSource {
    filename: String,
    directory: String,
    archive: String,
    index: u32,
}

impl TZipSource {
    pub fn new(backend: &ZipArchive, index: u32) -> Self {
        TZipSource {
            filename: backend.filename.clone(),
            directory: backend.directory.clone(),
            archive: backend.archive.clone(),
            index,
        }
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}
