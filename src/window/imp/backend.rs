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
    pub fn set_backend(&self, new_backend: Box<dyn Backend>, goto: Selection) {
        self.skip_loading.set(true);

        let w = self.widgets.get().unwrap();
        let parent_backend = w.backend.replace(new_backend);

        let new_backend = w.backend.borrow();
        dbg!(new_backend.class_name(), parent_backend.class_name());
        new_backend.set_parent(parent_backend);

        let new_store = new_backend.store();
        let current_sort = self.current_sort.clone();
        let last_sort = self.last_sort.clone();

        let sort = match current_sort.get() {
            Some(sort) => sort,
            None => {
                let sort = last_sort.get();
                self.current_sort.set(Some(sort));
                sort
            }
        };
        new_store.set_sort_column_id(sort.column, sort.order);

        new_store.connect_sort_column_changed(move |model| {
            Sort::on_sort_column_changed(model, &current_sort, &last_sort);
        });

        let path = Path::new(new_backend.path());
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        self.obj().set_title(&format!("MView6 - {filename}"));

        w.file_list_view.set_model(Some(&new_store));
        self.skip_loading.set(false);
        w.file_list_view.goto(&goto);
    }
}
