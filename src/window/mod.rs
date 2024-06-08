mod imp;

use gtk::glib;

use crate::application::MviewApplication;

glib::wrapper! {
    pub struct MViewWindow(ObjectSubclass<imp::MViewWindowSub>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl MViewWindow {
    pub fn new(app: &MviewApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}
