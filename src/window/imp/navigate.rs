use std::path::Path;

use super::MViewWindowImp;

use crate::{
    backends::Backend,
    filelistview::{Direction, FileListViewExt, Filter, Selection},
};
use eog::ScrollViewExt;
use gio::File;
use gtk::{prelude::*, TreePath, TreeViewColumn};

impl MViewWindowImp {
    pub(super) fn on_cursor_changed(&self) {
        let w = self.widgets.get().unwrap();
        if !self.skip_loading.get() {
            if let Some(current) = w.file_list_view.current() {
                let image = w.backend.borrow().image(w, &current);
                if w.backend.borrow().is_thumbnail() {
                    w.eog.set_image_pre(&image);
                    // w.eog.set_image_post();
                } else {
                    w.eog.set_image(&image);
                }
            }
        }
    }

    pub(super) fn on_row_activated(&self, _path: &TreePath, _column: &TreeViewColumn) {
        println!("on_row_activated");
        self.dir_enter();
    }

    pub fn dir_enter(&self) {
        let w = self.widgets.get().unwrap();
        if let Some(current) = w.file_list_view.current() {
            self.hop_parent_sort.set(Some(self.last_sort.get()));
            let backend = w.backend.borrow();
            let new_backend = backend.enter(&current);
            drop(backend);
            if let Some(new_backend) = new_backend {
                self.set_backend(new_backend, Selection::None);
            }
        }
    }

    pub fn dir_leave(&self) {
        self.hop_parent_sort.set(None);
        let w = self.widgets.get().unwrap();
        let backend = w.backend.borrow();
        let (new_backend, selection) = backend.leave();
        dbg!(&backend, &new_backend);
        drop(backend);
        self.set_backend(new_backend, selection);
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
        self.set_backend(new_backend, Selection::Name(filename.to_string()));
    }

    pub fn hop(&self, direction: Direction) {
        // dbg!("hop", &direction);
        if let Some(hop_parent_sort) = self.hop_parent_sort.get() {
            // remember current sort (last_stort), restore parent sort (hop_parent_sort)
            let last_sort = self.last_sort.get();
            self.last_sort.set(hop_parent_sort);
            self.current_sort.set(None);
            self.dir_leave();
            // navigate in parent
            let w = self.widgets.get().unwrap();
            w.file_list_view.navigate(direction, Filter::Container, 1);
            // enter dir with remembered sort (last_sort)
            self.last_sort.set(last_sort);
            self.current_sort.set(None);
            self.dir_enter();
            // dir_enter overwrites hop_parent_sort (with last_sort), so restore
            self.hop_parent_sort.set(Some(hop_parent_sort));
        }
    }
}