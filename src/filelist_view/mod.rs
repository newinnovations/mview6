mod imp;

use gtk::glib;

glib::wrapper! {
pub struct FileListView(ObjectSubclass<imp::FileListView>)
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
