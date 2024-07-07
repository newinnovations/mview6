use std::path::Path;

use super::MViewWindowImp;

use crate::{backends::Backend, filelistview::FileListViewExt};
use eog::ScrollViewExt;
use gio::File;
use gtk::{prelude::*, TreePath, TreeViewColumn};

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
            drop(backend);
            self.set_backend(new_backend, None);
        }
    }

    pub fn dir_leave(&self) {
        let w = self.widgets.get().unwrap();
        let backend = w.backend.borrow();
        let (new_backend, current_dir) = backend.leave();
        drop(backend);
        self.set_backend(new_backend, Some(&current_dir));
    }

    pub fn navigate_to(&self, file: &File) {
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
        let new_backend = <dyn Backend>::new(directory);
        self.set_backend(new_backend, Some(filename));
    }
}
