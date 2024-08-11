use std::{cell::Cell, fmt::Display};

use gtk4::{prelude::TreeSortableExtManual, ListStore, SortColumn, SortType};

use super::model::Columns;

#[derive(Clone, Copy, Debug, Default)]
pub enum Sort {
    Sorted((SortColumn, SortType)),
    #[default]
    Unsorted,
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sort::Sorted((c, t)) => write!(f, "{}", Sort::to_str(c, t)),
            Sort::Unsorted => write!(f, "Sort(none)"),
        }
    }
}

impl Sort {
    pub fn new(column: SortColumn, order: SortType) -> Self {
        Sort::Sorted((column, order))
    }

    pub fn sort_on_category() -> Self {
        Sort::new(SortColumn::Index(Columns::Cat as u32), SortType::Ascending)
    }

    pub fn on_sort_column_changed(model: &ListStore, current_sort: &Cell<Sort>) {
        let previous_sort = current_sort.get();
        if let Some((new_column, new_order)) = model.sort_column_id() {
            current_sort.set(Sort::new(new_column, new_order));
            if let Sort::Sorted((previous_column, _)) = previous_sort {
                if !previous_column.eq(&new_column) {
                    if let SortColumn::Index(4) = &new_column {
                        model.set_sort_column_id(
                            SortColumn::Index(Columns::Modified as u32),
                            SortType::Descending,
                        )
                    }
                }
            }
        }
    }

    pub fn to_str(col: &SortColumn, order: &SortType) -> String {
        format!(
            "Sort({}, {})",
            match *col {
                SortColumn::Default => "default".to_string(),
                SortColumn::Index(i) => format!("{}", i),
            },
            match *order {
                SortType::Ascending => "asc",
                _ => "des",
            }
        )
    }
}
