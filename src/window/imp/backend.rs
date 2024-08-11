use std::path::Path;

use glib::{clone, subclass::types::ObjectSubclassExt};
use gtk4::prelude::{GtkWindowExt, TreeSortableExt, TreeSortableExtManual, TreeViewExt, WidgetExt};

use crate::{
    backends::{thumbnail::Thumbnail, Backend},
    filelistview::Selection,
    window::imp::Sort,
};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn set_backend(&self, new_backend: Box<dyn Backend>, goto: Selection, set_parent: bool) {
        let skip_loading = self.skip_loading.get();
        self.skip_loading.set(true);

        let w = self.widgets();
        let parent_backend = self.backend.replace(new_backend);
        let new_backend = self.backend.borrow();

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

        new_store.connect_sort_column_changed(clone!(
            #[weak(rename_to = this)]
            self,
            move |model| {
                Sort::on_sort_column_changed(model, &this.current_sort);
            }
        ));

        let path = Path::new(new_backend.path());
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        self.obj().set_title(Some(&format!("MView6 - {filename}")));

        w.file_list_view.set_model(Some(&new_store));
        self.skip_loading.set(skip_loading);
        w.file_list_view.goto(&goto);
    }

    pub fn update_thumbnail_backend(&self) {
        let w = self.widgets();
        let backend = self.backend.borrow();
        if backend.is_thumbnail() {
            if let Some(thumbnail) =
                Thumbnail::new(w.image_view.allocation(), 0, self.thumbnail_size.get())
            {
                let (parent_backend, _selection) = backend.leave();
                drop(backend);
                self.backend.replace(parent_backend);
                let startpage = thumbnail.startpage();
                let new_backend = <dyn Backend>::thumbnail(thumbnail);
                self.set_backend(new_backend, startpage, true);
            }
        }
    }
}
