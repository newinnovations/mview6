use std::{cell::Cell, rc::Rc};

use gtk::{prelude::TreeSortableExtManual, ListStore, SortColumn, SortType};

use super::Columns;

#[derive(Clone, Copy, Debug)]
pub struct Sort {
    pub column: SortColumn,
    pub order: SortType,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            column: SortColumn::Index(Columns::Cat as u32),
            order: SortType::Ascending,
        }
    }
}

impl Sort {
    pub fn on_sort_column_changed(
        model: &ListStore,
        current_sort: &Rc<Cell<Option<Sort>>>,
        last_sort: &Rc<Cell<Sort>>,
    ) {
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
    }
}
