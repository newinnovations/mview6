use std::path::Path;

use super::MViewWindowImp;

use crate::filelistview::FileListViewExt;
use eog::{Image, ImageData, ImageExt, Job, ScrollViewExt};
use gio::File;
use gtk::prelude::*;

impl MViewWindowImp {
    pub(super) fn on_cursor_changed(&self) {
        let w = self.widgets.get().unwrap();
        if let Some(filename) = w.file_list_view.current_filename() {
            println!("Selected file {}", filename);
            let path = format!("{0}/{filename}", w.file_list.borrow().directory);
            println!("Path = {}", path);
            let file = File::for_path(path);
            self.load(&file);
        }
    }

    pub fn load(&self, file: &File) {

        if self.skip_loading.get() {
            println!("Skipping load");
            return;
        }

        let w = self.widgets.get().unwrap();
        let filename = file
            .path()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        let current = self.current_file.borrow();
        if current.eq(&filename) {
            println!("File {filename} already loaded, skipping");
            return;
        }
        drop(current);
        let image = Image::new_file(file, &filename);
        let filename_c = filename.clone();
        image.add_weak_ref_notify(move || {
            println!("**image [{filename_c}] disposed**");
        });
        match image.load(ImageData::IMAGE, None::<Job>.as_ref()) {
            Ok(()) => {
                w.eog.set_image(&image);
                self.current_file.replace(filename);
            }
            Err(error) => {
                println!("Error {}", error);
            }
        }
    }

    pub fn navigate_to(&self, file: &File) {
        let w = self.widgets.get().unwrap();
        let path = file.path().unwrap_or_default().clone();
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let directory = path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_str()
            .unwrap_or("/");
        println!("filename = {filename}");
        println!("directory = {directory}");
        let mut filelist = w.file_list.borrow_mut();
        let newstore = filelist.goto(directory);
        drop(filelist);
        if newstore.is_some() {
            w.file_list_view.set_model(newstore.as_ref());
            w.file_list_view.goto(filename);
        }
    }
}
