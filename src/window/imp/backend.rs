use gtk::{
    prelude::{TreeSortableExt, TreeSortableExtManual, TreeViewExt},
    SortColumn, SortType,
};

use crate::{
    backends::{Backend, Columns},
    filelistview::FileListViewExt,
    window::imp::Sort,
};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn set_backend(&self, new_backend: Box<dyn Backend>, goto: Option<&str>) {
        self.skip_loading.set(true);

        let new_store = new_backend.store();
        let current_sort = self.current_sort.clone();
        let last_sort = self.last_sort.clone();

        let w = self.widgets.get().unwrap();
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
            let new_sort = model
                .sort_column_id()
                .map(|(column, order)| Sort { column, order });
            let cur_sort = current_sort.get();
            let new_col = new_sort.map(|sort| sort.column);
            let cur_col = cur_sort.map(|sort| sort.column);
            println!("SortChange {:?} --> {:?}", cur_sort, new_sort);
            current_sort.set(new_sort);
            if let Some(sort) = new_sort {
                last_sort.set(sort);
            }
            if !cur_col.eq(&new_col) {
                println!("-- col changed {:?} --> {:?}", cur_col, new_col);
                if let Some(SortColumn::Index(4)) = &new_col {
                    // println!("-- changing modified sort to descending");
                    model.set_sort_column_id(
                        SortColumn::Index(Columns::Modified as u32),
                        SortType::Descending,
                    )
                }
            }
        });

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