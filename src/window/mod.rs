mod imp;

use gio::File;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib;

use crate::application::MviewApplication;

glib::wrapper! {
    pub struct MViewWindow(ObjectSubclass<imp::MViewWindowImp>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl MViewWindow {
    pub fn new(app: &MviewApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    // pub fn load(&self, file: &File) {
    //     self.imp().load(file);
    // }

    pub fn navigate_to(&self, file: &File) {
        self.imp().navigate_to(file);
    }
}
