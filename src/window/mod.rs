mod imp;

use std::cell::RefCell;

use gio::File;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::{glib, ScrolledWindow};

use crate::{
    application::MviewApplication,
    backends::{thumbnail::Message, Backend},
    filelistview::FileListView,
    image_view::ImageView,
};

glib::wrapper! {
    pub struct MViewWindow(ObjectSubclass<imp::MViewWindowImp>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

#[derive(Debug)]
pub struct MViewWidgets {
    hbox: gtk::Box,
    files_widget: ScrolledWindow,
    file_list_view: FileListView,
    backend: RefCell<Box<dyn Backend>>,
    // eog: ScrollView,
    eog: ImageView,
    pub sender: glib::Sender<Message>,
}

impl MViewWindow {
    pub fn new(app: &MviewApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn navigate_to(&self, file: &File) {
        self.imp().navigate_to(file);
    }
}
