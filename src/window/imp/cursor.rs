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

    pub fn set_backend(&self, new_backend: Box<dyn Backend>, goto: Option<&str>) {
        let new_store = new_backend.create_store();
        let sc = self.sort_column.clone();
        if new_store.is_some() {
            let new_store = new_store.unwrap();

            if let Some((sc, st)) = sc.get() {
                new_store.set_sort_column_id(sc, st);
            }

            new_store.connect_sort_column_changed(move |model| {
                let new_sc_st = model.sort_column_id();
                let cur_sc_st = sc.get();
                let new_sc = new_sc_st.map(|(sc, _)| sc);
                let cur_sc = cur_sc_st.map(|(sc, _)| sc);
                // println!("SortChange {:?} --> {:?}", cur_sc_st, new_sc_st);
                let col_changed = !cur_sc.eq(&new_sc);
                sc.set(new_sc_st);
                if col_changed {
                    // println!("-- col changed {:?} --> {:?}", cur_sc, new_sc);
                    if let Some(SortColumn::Index(4)) = &new_sc {
                        // println!("-- changing modified sort to descending");
                        model.set_sort_column_id(
                            SortColumn::Index(Columns::Modified as u32),
                            SortType::Descending,
                        )
                    }
                }
            });
            self.skip_loading.set(true);
            let w = self.widgets.get().unwrap();
            w.backend.replace(new_backend);
            w.file_list_view.set_model(Some(&new_store));
            self.skip_loading.set(false);
            match goto {
                Some(name) => {
                    w.file_list_view.goto(name);
                }
                None => {
                    w.file_list_view.goto_first();
                }
            }
        }
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
