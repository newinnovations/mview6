use gtk::{
    prelude::{TreeSortableExt, TreeSortableExtManual, TreeViewExt},
    SortColumn, SortType,
};

use crate::{
    backends::{Backend, Columns},
    filelistview::FileListViewExt,
};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn set_backend(&self, new_backend: Box<dyn Backend>, goto: Option<&str>) {
        let new_store = new_backend.store();
        let sc = self.sort_column.clone();
        // if new_store.is_some() {
        //     let new_store = new_store.unwrap();

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
        // }
    }
}
