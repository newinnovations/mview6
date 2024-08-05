mod imp;

use crate::application::MviewApplication;
use gio::File;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib;
pub use imp::MViewWidgets;

glib::wrapper! {
    pub struct MViewWindow(ObjectSubclass<imp::MViewWindowImp>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl MViewWindow {
    pub fn new(app: &MviewApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn navigate_to(&self, file: &File, set_parent:bool) {
        self.imp().navigate_to(file, set_parent);
    }
}
