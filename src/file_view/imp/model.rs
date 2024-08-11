use gtk4::{prelude::TreeSortableExtManual, ListStore};

use super::cursor::TreeModelMviewExt;
use crate::backends::thumbnail::TReference;

#[derive(Debug)]
#[repr(i32)]
pub enum Direction {
    Up = 0,
    Down,
}

#[derive(Debug)]
#[repr(i32)]
pub enum Filter {
    None = 0,
    Image,
    Favorite,
    Container,
}

#[derive(Debug)]
#[repr(u32)]
pub enum Columns {
    Cat = 0,
    Icon,
    Name,
    Size,
    Modified,
    Index,
    Folder,
}

impl Columns {
    pub fn store() -> ListStore {
        let col_types: [glib::Type; 7] = [
            glib::Type::U32,
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::U64,
            glib::Type::U64,
            glib::Type::U32,
            glib::Type::STRING,
        ];
        let store = ListStore::new(&col_types);
        store.set_sort_func(
            gtk4::SortColumn::Index(Columns::Cat as u32),
            |model, iter1, iter2| {
                let cat1 = model.category_id(iter1);
                let cat2 = model.category_id(iter2);
                let result = cat1.cmp(&cat2);
                if result.is_eq() {
                    let filename1 = model.name(iter1).to_lowercase();
                    let filename2 = model.name(iter2).to_lowercase();
                    filename1.cmp(&filename2)
                } else {
                    result
                }
                .into()
            },
        );
        store
    }
}

#[derive(Debug)]
pub enum Selection {
    Name(String),
    Index(u32),
    None,
}

impl From<TReference> for Selection {
    fn from(item: TReference) -> Self {
        match item {
            TReference::FileReference(file) => Selection::Name(file.filename()),
            TReference::ZipReference(zip) => Selection::Index(zip.index()),
            TReference::RarReference(rar) => Selection::Name(rar.selection()),
            TReference::None => Selection::None,
        }
    }
}
