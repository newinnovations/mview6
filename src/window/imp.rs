mod cursor;
mod keyboard;

use crate::{
    backends::{filesystem::FileSystem, Backend, Columns},
    draw::draw,
    filelist::FileList,
    filelistview::{FileListView, FileListViewExt},
};
use eog::{ScrollView, ScrollViewExt};
use gdk_pixbuf::PixbufLoader;
use glib::{clone, once_cell::unsync::OnceCell};
use gtk::{glib, prelude::*, subclass::prelude::*, ScrolledWindow, SortColumn, SortType};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Debug)]
struct MViewWidgets {
    hbox: gtk::Box,
    files_widget: ScrolledWindow,
    file_list: Rc<RefCell<FileList>>,
    file_list_view: FileListView,
    backend: RefCell<Box<dyn Backend>>,
    eog: ScrollView,
}

// #[derive(Debug, Default)]
#[derive(Default)]
pub struct MViewWindowImp {
    widgets: OnceCell<MViewWidgets>,
    full_screen: Cell<bool>,
    skip_loading: Cell<bool>,
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
        self.full_screen.set(false);
        self.skip_loading.set(false);

        let window = self.obj();

        // let a = window.clone().upcast::<gtk::Window>().accessible();

        window.set_title("MView6");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(1280, 720);

        let loader = PixbufLoader::with_type("svg").unwrap();
        loader.write(include_bytes!("icon.svg")).unwrap();
        loader.close().unwrap();
        window.set_icon(Some(&loader.pixbuf().unwrap()));

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        window.add(&hbox);

        let files_widget = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        files_widget.set_shadow_type(gtk::ShadowType::EtchedIn);
        files_widget.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        hbox.add(&files_widget);

        let file_list = Rc::new(RefCell::new(FileList::new("/home/martin/Pictures")));
        let file_list_view = FileListView::new();
        file_list_view.set_model(file_list.borrow().read().as_ref());
        file_list_view.set_vexpand(true);
        file_list_view.set_sort_column(SortColumn::Index(Columns::Cat as u32), SortType::Ascending);
        files_widget.add(&file_list_view);

        let eog = ScrollView::new();
        eog.add_weak_ref_notify(|| {
            println!("**eog::ScrollView disposed**");
        });
        eog.set_scroll_wheel_zoom(true);
        eog.set_zoom_mode(eog::ZoomMode::Max);
        hbox.add(&eog);

        let backend = RefCell::new(Box::new(FileSystem::new("init")));

        // let f = gio::File::for_path("/home/martin/Pictures/mview-logo.jpg");
        // let img = Image::new_file(&f, "welcome");
        // img.add_weak_ref_notify(move || {
        //     println!("**welcome image disposed**");
        // });
        // let result = img.load(ImageData::IMAGE, None::<Job>.as_ref());

        let result = draw("MView6 - Advanced Image Viewer");
        if result.is_ok() {
            let img = result.unwrap();
            img.add_weak_ref_notify(move || {
                println!("**welcome image disposed**");
            });
            eog.set_image(&img);
        }

        // match result {
        //     Ok(img) => {
        //         println!("OK");
        //         let jpg = img.is_jpeg();
        //         println!("is jpg {}", jpg);

        //         let (width, height) = img.size();
        //         println!("Size {} {}", width, height);

        //         eog.set_image(&img);
        //     }
        //     Err(error) => {
        //         println!("Error {}", error);
        //     }
        // }

        window.connect_key_press_event(clone!(@weak self as imp => @default-panic, move |_, e| {
            imp.on_key_press(e);
            glib::Propagation::Stop
        }));

        file_list_view.connect_cursor_changed(clone!(@weak self as imp => move |_| {
            imp.on_cursor_changed();
        }));

        file_list_view.connect_row_activated(clone!(@weak self as imp => move |_, path, column| {
            imp.on_row_activated(path, column);
        }));

        self.widgets
            .set(MViewWidgets {
                hbox,
                file_list,
                backend,
                file_list_view,
                files_widget,
                eog,
            })
            .expect("Failed to initialize MView window");

        window.show_all();

        // self.widgets.get().unwrap().eog.set_offset(0, 0);

        println!("MViewWindowSub: constructed done");
    }
}

impl WidgetImpl for MViewWindowImp {}
impl ContainerImpl for MViewWindowImp {}
impl BinImpl for MViewWindowImp {}
impl WindowImpl for MViewWindowImp {}
impl ApplicationWindowImpl for MViewWindowImp {}
