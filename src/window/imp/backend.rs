use std::path::Path;

use glib::subclass::types::ObjectSubclassExt;
use gtk::prelude::{GtkWindowExt, TreeSortableExt, TreeSortableExtManual, TreeViewExt};

use crate::{
    backends::Backend,
    filelistview::{FileListViewExt, Selection},
    window::imp::Sort,
};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn set_backend(&self, new_backend: Box<dyn Backend>, goto: Selection, set_parent: bool) {
        let skip_loading = self.skip_loading.get();
        self.skip_loading.set(true);

        let w = self.widgets.get().unwrap();
        let parent_backend = w.backend.replace(new_backend);
        let new_backend = w.backend.borrow();

        if set_parent {
            // dbg!(new_backend.class_name(), parent_backend.class_name());
            parent_backend.set_sort(&self.current_sort.get());
            if !parent_backend.is_bookmarks() {
                new_backend.set_parent(parent_backend);
            }
        }

        let new_sort = match new_backend.sort() {
            Sort::Sorted(sort) => {
                let sort = Sort::Sorted(sort);
                self.current_sort.set(sort);
                sort
            }
            Sort::Unsorted => {
                let sort = self.current_sort.get();
                new_backend.set_sort(&sort);
                sort
            }
        };

        let new_store = new_backend.store();
        match new_sort {
            Sort::Sorted((column, order)) => new_store.set_sort_column_id(column, order),
            Sort::Unsorted => (),
        };

        let current_sort = self.current_sort.clone();
        new_store.connect_sort_column_changed(move |model| {
            Sort::on_sort_column_changed(model, &current_sort);
        });

        let path = Path::new(new_backend.path());
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        self.obj().set_title(&format!("MView6 - {filename}"));

        w.file_list_view.set_model(Some(&new_store));
        self.skip_loading.set(skip_loading);
        w.file_list_view.goto(&goto);
    }
}
