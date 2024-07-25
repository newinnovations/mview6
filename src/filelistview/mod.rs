mod imp;

use glib::{Cast, IsA};
use gtk::{
    glib,
    prelude::{GtkListStoreExtManual, TreeModelExt, TreeSortableExtManual, TreeViewExt},
    ListStore, SortColumn, SortType, TreeIter, TreePath, TreeView, TreeViewColumn,
};

use crate::{
    backends::{Columns, Selection, TreeModelMviewExt},
    category::Category,
};

glib::wrapper! {
pub struct FileListView(ObjectSubclass<imp::FileListViewImp>)
    @extends gtk::Container, gtk::Widget, gtk::TreeView, gtk::Scrollable;
}

impl FileListView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for FileListView {
    fn default() -> Self {
        Self::new()
    }
}

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

pub struct Cursor {
    store: ListStore,
    iter: TreeIter,
    position: i32,
}

impl Cursor {
    pub fn new(store: ListStore, iter: TreeIter, position: i32) -> Self {
        Cursor {
            store,
            iter,
            position,
        }
    }

    /// Postion in the list (depends on the sorting order)
    pub fn position(&self) -> i32 {
        self.position
    }

    /// Value of the index field of the row
    pub fn index(&self) -> u32 {
        self.store.index(&self.iter)
    }

    /// Value of the name field of the row
    pub fn name(&self) -> String {
        self.store.name(&self.iter)
    }

    /// Value of the folder field of the row
    pub fn folder(&self) -> String {
        self.store.folder(&self.iter)
    }

    /// Value of the category field of the row (as u32)
    pub fn category_id(&self) -> u32 {
        self.store.category_id(&self.iter)
    }

    /// Value of the category field of the row (as Category)
    pub fn category(&self) -> Category {
        self.store.category(&self.iter)
    }

    pub fn store_size(&self) -> i32 {
        self.store.iter_n_children(None)
    }

    pub fn update(&self, new_category: Category, new_filename: &str) {
        self.store.set(
            &self.iter,
            &[
                (Columns::Cat as u32, &new_category.id()),
                (Columns::Icon as u32, &new_category.icon()),
                (Columns::Name as u32, &new_filename),
            ],
        );
    }

    pub fn set_sort(&self, sort_column_id: SortColumn, order: SortType) {
        self.store.set_sort_column_id(sort_column_id, order);
    }

    pub fn set_sort_column(&self, new_column: SortColumn) {
        let current_sort = self.store.sort_column_id();
        let new_direction = match current_sort {
            Some((current_column, current_direction)) => {
                if current_column.eq(&new_column) {
                    match current_direction {
                        SortType::Ascending => SortType::Descending,
                        _ => SortType::Ascending,
                    }
                } else {
                    SortType::Ascending
                }
            }
            None => SortType::Ascending,
        };
        self.store.set_sort_column_id(new_column, new_direction);
    }

    fn navigate(&self, direction: Direction, filter: Filter, count: i32) -> Option<TreePath> {
        let mut cnt = count;
        loop {
            let last = self.iter;
            let result = match direction {
                Direction::Up => self.store.iter_previous(&self.iter),
                Direction::Down => self.store.iter_next(&self.iter),
            };
            if !result {
                if count != cnt {
                    return self.store.path(&last);
                }
                return None;
            }

            let cat = self.store.category(&self.iter);

            let skip = match filter {
                Filter::None => false,
                Filter::Image => cat != Category::Image && cat != Category::Favorite,
                Filter::Favorite => cat != Category::Favorite,
                Filter::Container => cat != Category::Direcory && cat != Category::Archive,
            };

            if skip {
                continue;
            }

            cnt -= 1;
            if cnt == 0 {
                break;
            }
        }
        self.store.path(&self.iter)
    }

    pub fn next(&self) -> bool {
        self.store.iter_next(&self.iter)
    }
}

pub trait FileListViewExt: IsA<FileListView> + IsA<TreeView> + 'static {
    fn goto(&self, selection: &Selection) -> bool;
    fn current(&self) -> Option<Cursor>;
    fn navigate(&self, direction: Direction, filter: Filter, count: i32);
    // fn set_sort_column(&self, sort_column_id: SortColumn, order: SortType);
    fn set_unsorted(&self);
}

impl<O: IsA<FileListView> + IsA<TreeView>> FileListViewExt for O {
    fn current(&self) -> Option<Cursor> {
        let (tree_path, _) = self.cursor();
        let store = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(path) = tree_path {
            store.iter(&path).map(|iter| Cursor {
                store,
                iter,
                position: *path.indices().first().unwrap_or(&0),
            })
        } else {
            store.iter_first().map(|iter| Cursor {
                store,
                iter,
                position: 0,
            })
        }
    }

    fn goto(&self, selection: &Selection) -> bool {
        println!("Goto {:?}", selection);
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(iter) = model.iter_first() {
            loop {
                let found = match selection {
                    Selection::Name(filename) => *filename == model.name(&iter),
                    Selection::Index(index) => *index == model.index(&iter),
                    Selection::None => true,
                };
                if found {
                    let tp = model.path(&iter).unwrap_or_default();
                    self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                    return true;
                }
                if !model.iter_next(&iter) {
                    return false;
                }
            }
        } else {
            false
        }
    }

    fn navigate(&self, direction: Direction, filter: Filter, count: i32) {
        if let Some(current) = self.current() {
            if let Some(tree_path) = current.navigate(direction, filter, count) {
                self.set_cursor(&tree_path, None::<&TreeViewColumn>, false);
            }
        }
    }

    // fn set_sort_column(&self, sort_column_id: SortColumn, order: SortType) {
    //     let model = self.model().unwrap().downcast::<ListStore>().unwrap();
    //     model.set_sort_column_id(sort_column_id, order);
    // }

    fn set_unsorted(&self) {
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        model.set_unsorted();
    }
}
