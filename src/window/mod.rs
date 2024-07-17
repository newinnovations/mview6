mod imp;

use std::cell::RefCell;

use eog::ScrollView;
use gio::File;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::{glib, ScrolledWindow};
use image::DynamicImage;

use crate::{
    application::MviewApplication,
    backends::{filesystem::TFileSource, Backend},
    filelistview::FileListView,
};

glib::wrapper! {
    pub struct MViewWindow(ObjectSubclass<imp::MViewWindowImp>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

#[derive(Debug, Clone)]
pub enum TSource {
    FileSource(TFileSource),
    None,
}

#[derive(Debug, Clone, Default)]
pub struct TCommand {
    id: i32,
    tasks: Vec<TTask>,
}

impl TCommand {
    pub fn new(id: i32, tasks: Vec<TTask>) -> Self {
        TCommand { id, tasks }
    }
}

#[derive(Debug, Clone)]
pub struct TTask {
    size: u32,
    position: (i32, i32),
    source: TSource,
}

impl TTask {
    pub fn new(size: u32, x: i32, y: i32, source: TSource) -> Self {
        TTask {
            size,
            position: (x, y),
            source,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TResult {
    id: i32,
    task: TTask,
    image: Option<DynamicImage>,
}

impl TResult {
    pub fn new(id: i32, task: TTask, image: Option<DynamicImage>) -> Self {
        TResult { id, task, image }
    }
}

pub enum Message {
    Command(TCommand),
    Result(TResult),
    // Test(i32),
    UpdateLabel(String),
    Image(DynamicImage),
}

#[derive(Debug)]
pub struct MViewWidgets {
    hbox: gtk::Box,
    files_widget: ScrolledWindow,
    file_list_view: FileListView,
    backend: RefCell<Box<dyn Backend>>,
    eog: ScrollView,
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
