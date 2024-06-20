use std::path::Path;

use super::MViewWindowImp;

use crate::{
    backends::{Backend, Columns},
    filelistview::FileListViewExt,
};
use eog::ScrollViewExt;
use gio::File;
use gtk::{prelude::*, SortColumn, SortType, TreePath, TreeViewColumn};

impl MViewWindowImp {
    pub(super) fn on_cursor_changed(&self) {
        let w = self.widgets.get().unwrap();
        if let Some((model, iter)) = w.file_list_view.iter() {
            let image = w.backend.borrow().image(model, iter);
            w.eog.set_image(&image);
        }
    }

    pub(super) fn on_row_activated(&self, _path: &TreePath, _column: &TreeViewColumn) {
        println!("on_row_activated");
        self.dir_enter();
    }

    pub fn dir_enter(&self) {
        let w = self.widgets.get().unwrap();
        if let Some((model, iter)) = w.file_list_view.iter() {
            let backend = w.backend.borrow();
            let new_backend = backend.enter(model, iter);
            let new_store = new_backend.create_store();
            drop(backend);
            if new_store.is_some() {
                w.backend.replace(new_backend);
                self.skip_loading.set(true);
                w.file_list_view.set_model(new_store.as_ref());
                w.file_list_view
                    .set_sort_column(SortColumn::Index(Columns::Cat as u32), SortType::Ascending);
                self.skip_loading.set(false);
                w.file_list_view.goto_first();
            }
        }
    }

    pub fn dir_leave(&self) {
        let w = self.widgets.get().unwrap();
        let backend = w.backend.borrow();
        let (new_backend, current_dir) = backend.leave();
        let new_store = new_backend.create_store();
        drop(backend);
        if new_store.is_some() {
            w.backend.replace(new_backend);
            self.skip_loading.set(true);
            w.file_list_view.set_model(new_store.as_ref());
            w.file_list_view
                .set_sort_column(SortColumn::Index(Columns::Cat as u32), SortType::Ascending);
            self.skip_loading.set(false);
            w.file_list_view.goto(&current_dir);
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
        let backend = <dyn Backend>::new(directory);
        w.file_list_view.set_model(backend.create_store().as_ref());
        w.backend.replace(backend);
        w.file_list_view
            .set_sort_column(SortColumn::Index(Columns::Cat as u32), SortType::Ascending);
        w.file_list_view.goto(filename);
    }
}
