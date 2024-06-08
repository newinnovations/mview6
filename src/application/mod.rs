mod imp;

use gio::ApplicationFlags;
use gtk::{gio, glib};

glib::wrapper! {
    pub struct MviewApplication(ObjectSubclass<imp::MviewApplicationImp>)
        @extends gio::Application, gtk::Application;
}

impl MviewApplication {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "org.gtk-rs.SimpleApplication")
            .property("flags", ApplicationFlags::NON_UNIQUE.union(ApplicationFlags::HANDLES_OPEN))
            .build()
    }
}
