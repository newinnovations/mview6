use eog::Image;
use filesystem::FileSystem;
use glib::IsA;
use gtk::{
    prelude::{TreeModelExt, TreeSortableExtManual},
    ListStore, TreeIter, TreeModel,
};

use crate::{category::Category, filelistview::Direction};

pub mod archive_rar;
pub mod archive_zip;
pub mod filesystem;
pub mod invalid;

#[derive(Debug)]
#[repr(u32)]
pub enum Columns {
    Cat = 0,
    Icon,
    Name,
    Size,
    Modified,
}

pub trait Backend {
    fn class_name(&self) -> &str;
    fn create_store(&self) -> Option<ListStore>;
    fn favorite(&self, model: ListStore, iter: TreeIter, direction: Direction) -> bool;
    fn enter(&self, model: ListStore, iter: TreeIter) -> Box<dyn Backend>;
    fn leave(&self) -> (Box<dyn Backend>, String);
    fn image(&self, model: ListStore, iter: TreeIter) -> Image;
}

impl std::fmt::Debug for dyn Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Backend({})", self.class_name())
    }
}

impl dyn Backend {
    pub fn new(directory: &str) -> (Box<dyn Backend>, ListStore) {
        let backend = FileSystem::new(directory);
        let store = backend.create_store().unwrap(); //FIXME
        (Box::new(backend), store)
    }
}

pub trait TreeModelMviewExt: IsA<TreeModel> + 'static {
    fn filename(&self, iter: &TreeIter) -> String {
        self.value(iter, Columns::Name as i32)
            .get::<String>()
            .unwrap_or_default()
    }
    fn category(&self, iter: &TreeIter) -> u32 {
        self.value(iter, Columns::Cat as i32)
            .get::<u32>()
            .unwrap_or(Category::Unsupported.id())
    }
}

impl<O: IsA<TreeModel>> TreeModelMviewExt for O {}

pub fn empty_store() -> ListStore {
    let col_types: [glib::Type; 5] = [
        glib::Type::U32,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::U64,
        glib::Type::U64,
    ];
    let store = ListStore::new(&col_types);
    store.set_sort_func(
        gtk::SortColumn::Index(Columns::Cat as u32),
        |model, iter1, iter2| {
            let cat1 = model.category(iter1);
            let cat2 = model.category(iter2);
            let result = cat1.cmp(&cat2);
            if result.is_eq() {
                let filename1 = model.filename(iter1).to_lowercase();
                let filename2 = model.filename(iter2).to_lowercase();
                filename1.cmp(&filename2)
            } else {
                result
            }
        },
    );
    store
}
