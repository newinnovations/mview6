use std::path::Path;

use super::MViewWindowImp;

use crate::{
    backends::Backend,
    filelistview::{Direction, FileListViewExt, Filter, Selection, Sort},
};
use gio::File;
use gtk::{prelude::*, TreePath, TreeViewColumn};

impl MViewWindowImp {
    pub(super) fn on_cursor_changed(&self) {
        let w = self.widgets.get().unwrap();
        if !self.skip_loading.get() {
            if let Some(current) = w.file_list_view.current() {
                let image = w.backend.borrow().image(w, &current);
                if w.backend.borrow().is_thumbnail() {
                    w.eog.set_image_pre(image);
                    // w.eog.set_image_post();
                } else {
                    w.eog.set_image(image);
                }
            }
        }
    }

    pub(super) fn on_row_activated(&self, _path: &TreePath, _column: &TreeViewColumn) {
        println!("on_row_activated");
        self.dir_enter(None);
    }

    pub fn dir_enter(&self, force_sort: Option<Sort>) {
        let w = self.widgets.get().unwrap();
        if let Some(current) = w.file_list_view.current() {
            let backend = w.backend.borrow();
            let new_backend = backend.enter(&current);
            drop(backend);
            if let Some(new_backend) = new_backend {
                if let Some(sort) = force_sort {
                    new_backend.set_sort(&sort);
                }
                self.set_backend(new_backend, Selection::None, true);
            }
        }
    }

    pub fn dir_leave(&self) {
        let w = self.widgets.get().unwrap();
        let backend = w.backend.borrow();
        let (new_backend, selection) = backend.leave();
        // dbg!(&backend, &new_backend);
        drop(backend);
        self.set_backend(new_backend, selection, false);
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
        dbg!(filename, directory);
        let new_backend = <dyn Backend>::new(directory);
        self.set_backend(new_backend, Selection::Name(filename.to_string()), true);
    }

    pub fn hop(&self, direction: Direction) {
        let active_sort = self.current_sort.get();
        let w = self.widgets.get().unwrap();

        // goto and navigate in parent
        self.skip_loading.set(true);
        self.dir_leave();
        w.file_list_view.navigate(direction, Filter::Container, 1);

        // enter dir with remembered sort
        self.skip_loading.set(false);
        self.dir_enter(Some(active_sort));
    }
}
