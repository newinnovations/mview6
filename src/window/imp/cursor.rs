use std::{fs, path::Path};

use super::MViewWindowImp;

use crate::{backends::Columns, category::Category, draw::draw, filelistview::FileListViewExt};
use eog::{Image, ImageData, ImageExt, Job, ScrollViewExt};
use gio::File;
use gtk::{prelude::*, SortColumn, SortType, TreePath, TreeViewColumn};

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

    pub(super) fn on_row_activated(&self, _path: &TreePath, _column: &TreeViewColumn) {
        println!("on_row_activated");
        self.dir_enter();
    }

    pub fn dir_enter(&self) {
        let w = self.widgets.get().unwrap();
        if let Some(subdir) = &w.file_list_view.current_filename() {
            let mut filelist = w.file_list.borrow_mut();
            let newstore = filelist.enter(subdir);
            drop(filelist);
            if newstore.is_some() {
                self.skip_loading.set(true);
                w.file_list_view.set_model(newstore.as_ref());
                w.file_list_view
                    .set_sort_column(SortColumn::Index(Columns::Cat as u32), SortType::Ascending);
                self.skip_loading.set(false);
                w.file_list_view.goto_first();
            }
        }
    }

    pub fn dir_leave(&self) {
        let w = self.widgets.get().unwrap();
        let mut filelist = w.file_list.borrow_mut();
        let newstore = filelist.leave();
        drop(filelist);
        if newstore.is_some() {
            let (newstore, current_dir) = newstore.unwrap();
            self.skip_loading.set(true);
            w.file_list_view.set_model(Some(&newstore));
            w.file_list_view
                .set_sort_column(SortColumn::Index(Columns::Cat as u32), SortType::Ascending);
            self.skip_loading.set(false);
            w.file_list_view.goto(&current_dir);
        }
    }

    pub fn load(&self, file: &File) {
        if self.skip_loading.get() {
            println!("Skipping load");
            return;
        }

        let path = file.path().unwrap_or_default();

        let filename = path.to_str().unwrap_or_default().to_string();

        let cat = match fs::metadata(&path) {
            Ok(metadata) => Category::determine(&filename, &metadata),
            Err(_) => Category::Unsupported,
        };

        let current = self.current_file.borrow();
        if current.eq(&filename) {
            println!("File {filename} already loaded, skipping");
            return;
        }
        drop(current);

        let image = match cat {
            Category::Direcory | Category::Archive | Category::Unsupported => {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                draw(name)
            }
            _ => Ok(Image::new_file(file, &filename)),
        };

        if image.is_ok() {
            let image = image.unwrap();
            let filename_c = filename.clone();
            image.add_weak_ref_notify(move || {
                println!("**image [{filename_c}] disposed**");
            });

            let w = self.widgets.get().unwrap();

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
