mod cursor;
mod keyboard;

use eog::{Image, ImageData, ImageExt, ImageExtManual, Job, ScrollView, ScrollViewExt};
use glib::{clone, once_cell::unsync::OnceCell};
use gtk::{glib, prelude::*, subclass::prelude::*, Box, ScrolledWindow};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::filelist::FileList;
use crate::filelistview::FileListView;

#[derive(Debug)]
struct MViewWidgets {
    hbox: Box,
    file_list: Rc<RefCell<FileList>>,
    file_window: ScrolledWindow,
    sv: ScrollView,
    treeview: FileListView,
}

#[derive(Debug, Default)]
pub struct MViewWindowImp {
    widgets: OnceCell<MViewWidgets>,
    fs: Cell<bool>,
    current_file: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for MViewWindowImp {
    const NAME: &'static str = "MViewWindow";
    type Type = super::MViewWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for MViewWindowImp {
    fn constructed(&self) {
        self.parent_constructed();
        self.fs.set(false);

        let window = self.obj();

        // let a = window.clone().upcast::<gtk::Window>().accessible();

        window.set_title("MView6");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(1280, 720);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        window.add(&hbox);

        let file_window = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        file_window.set_shadow_type(gtk::ShadowType::EtchedIn);
        file_window.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        hbox.add(&file_window);

        let file_list = Rc::new(RefCell::new(FileList::new("/home/martin/Pictures")));
        let treeview = FileListView::new();
        treeview.set_model(file_list.borrow().read().as_ref());
        treeview.set_vexpand(true);
        // treeview.set_search_column(Columns::Name as i32);

        file_window.add(&treeview);

        let sv = ScrollView::new();
        sv.add_weak_ref_notify(|| {
            println!("ScrollView disposed");
        });
        sv.set_scroll_wheel_zoom(true);
        sv.set_zoom_mode(eog::ZoomMode::Max);
        hbox.add(&sv);

        let f = gio::File::for_path("/home/martin/Pictures/mview-logo.jpg");
        let img = Image::new_file(&f, "welcome");
        img.add_weak_ref_notify(move || {
            println!("**welcome image disposed**");
        });
        let result = img.load(ImageData::IMAGE, None::<Job>.as_ref());

        match result {
            Ok(()) => {
                println!("OK");
                let jpg = img.is_jpeg();
                println!("is jpg {}", jpg);

                let (width, height) = img.size();
                println!("Size {} {}", width, height);

                sv.set_image(&img);
            }
            Err(error) => {
                println!("Error {}", error);
            }
        }

        window.connect_key_press_event(clone!(@weak self as imp => @default-panic, move |_, e| {
            imp.on_key_press(e);
            glib::Propagation::Stop
        }));

        treeview.connect_cursor_changed(clone!(@weak self as imp => move |_| {
            imp.on_cursor_changed();
        }));

        self.widgets
            .set(MViewWidgets {
                file_list,
                file_window,
                hbox,
                sv,
                treeview,
            })
            .expect("Failed to initialize MView window");

        window.show_all();

        self.widgets.get().unwrap().sv.set_offset(0, 0);

        println!("MViewWindowSub: constructed done");

    }
}

impl WidgetImpl for MViewWindowImp {}
impl ContainerImpl for MViewWindowImp {}
impl BinImpl for MViewWindowImp {}
impl WindowImpl for MViewWindowImp {}
impl ApplicationWindowImpl for MViewWindowImp {}
