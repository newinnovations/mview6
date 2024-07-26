use gtk::{SortColumn, SortType};

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
