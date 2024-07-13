use std::path::Path;

use chrono::{Local, TimeZone};
use eog::Image;
use gtk::{prelude::GtkListStoreExtManual, ListStore, TreeIter};
use unrar::{error::UnrarError, Archive, UnrarResult};

use crate::{backends::empty_store, category::Category, draw::draw, loader::Loader};

use super::{filesystem::FileSystem, Backend, Columns, TreeModelMviewExt};

pub struct RarArchive {
    filename: String,
    store: ListStore,
}

impl RarArchive {
    pub fn new(filename: &str) -> Self {
        RarArchive {
            filename: filename.to_string(),
            store: Self::create_store(filename),
        }
    }

    fn create_store(filename: &str) -> ListStore {
        println!("create_store RarArchive {}", &filename);
        let store = empty_store();
        match list_rar(filename, &store) {
            Ok(()) => println!("OK"),
            Err(e) => println!("ERROR {:?}", e),
        };
        store
    }
}

impl Backend for RarArchive {
    fn class_name(&self) -> &str {
        "RarArchive"
    }

    fn path(&self) -> &str {
        &self.filename
    }

    fn store(&self) -> ListStore {
        self.store.clone()
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
        let file_size = entry.unpacked_size;
        let modified = unix_from_msdos(entry.file_time);
        if file_size == 0 {
            continue;
        }
        if cat.id() == Category::Unsupported.id() {
            continue;
        }
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

pub fn unix_from_msdos(dostime: u32) -> u64 {
    let second = (dostime & 0b0000000000011111) << 1;
    let minute = (dostime & 0b0000011111100000) >> 5;
    let hour = (dostime & 0b1111100000000000) >> 11;

    let datepart = dostime >> 16;
    let day = datepart & 0b0000000000011111;
    let month = (datepart & 0b0000000111100000) >> 5;
    let year = 1980 + ((datepart & 0b1111111000000000) >> 9);

    match Local.with_ymd_and_hms(year as i32, month, day, hour, minute, second) {
        chrono::offset::LocalResult::Single(datetime) => datetime.timestamp() as u64,
        _ => {
            println!("Could not create local datetime (Ambiguous or None)");
            0_u64
        }
    }
}
