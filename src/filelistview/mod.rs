mod imp;

use glib::{Cast, IsA};
use gtk::{
    glib,
    prelude::{GtkListStoreExtManual, TreeModelExt, TreeViewExt},
    ListStore, TreeIter, TreePath, TreeView, TreeViewColumn,
};

use crate::filelist::Columns;

glib::wrapper! {
pub struct FileListView(ObjectSubclass<imp::FileListViewImp>)
    @extends gtk::Container, gtk::Widget, gtk::TreeView, gtk::Scrollable;
}

impl FileListView {
    // pub fn new(app: &Application) -> Self {
    //     glib::Object::builder().property("application", app).build()
    // }
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

pub trait FileListViewExt: IsA<FileListView> + IsA<TreeView> + 'static {
    fn goto_first(&self);
    fn goto(&self, filename: &str) -> bool;
    fn iter(&self) -> Option<(ListStore, TreeIter)>;
    fn filename(&self) -> Option<String>;
    fn write(&self);
}

impl<O: IsA<FileListView> + IsA<TreeView>> FileListViewExt for O {
    fn goto_first(&self) {
        let tp = TreePath::from_indicesv(&[0]);
        self.set_cursor(&tp, None::<&TreeViewColumn>, false);
    }

    fn iter(&self) -> Option<(ListStore, TreeIter)> {
        let (tp, _) = self.cursor();
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(path) = tp {
            if let Some(iter) = model.iter(&path) {
                Some((model, iter))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn filename(&self) -> Option<String> {
        if let Some((model, iter)) = self.iter() {
            let filename = model
                .value(&iter, Columns::Name as i32)
                .get::<String>()
                .unwrap_or("none".to_string());
            Some(filename)
        } else {
            None
        }
    }

    fn write(&self) {
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        let iter = model.iter_first().unwrap();
        // model.set_value(&iter, Columns::Name as u32, &Value::from("xxx"));
        let c = 100 as u32;
        model.set(
            &iter,
            &[(Columns::Cat as u32, &c), (Columns::Name as u32, &"blah")],
        )
    }

    fn goto(&self, filename: &str) -> bool {
        println!("Goto {filename}");
        let model = self.model().unwrap().downcast::<ListStore>().unwrap();
        if let Some(iter) = model.iter_first() {
            loop {
                let entry = model
                    .value(&iter, Columns::Name as i32)
                    .get::<String>()
                    .unwrap_or("none".to_string());
                if entry == filename {
                    let tp = model.path(&iter).unwrap_or_default();
                    self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                    return true;
                }
                if !model.iter_next(&iter) {
                    return false;
                }
            }
        }
        return false;
    }
}
