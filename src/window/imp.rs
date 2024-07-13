mod backend;
mod cursor;
mod keyboard;

use crate::{
    backends::{Backend, Columns},
    filelistview::FileListView,
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
    file_list_view: FileListView,
    backend: RefCell<Box<dyn Backend>>,
    eog: ScrollView,
}

#[derive(Clone, Copy, Debug)]
struct Sort {
    column: SortColumn,
    order: SortType,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            column: SortColumn::Index(Columns::Cat as u32),
            order: SortType::Ascending,
        }
    }
}

#[derive(Debug, Default)]
pub struct MViewWindowImp {
    widgets: OnceCell<MViewWidgets>,
    full_screen: Cell<bool>,
    skip_loading: Cell<bool>,
    current_sort: Rc<Cell<Option<Sort>>>,
    last_sort: Rc<Cell<Sort>>, // current_sort or sort before unsorted (through favorite op)
    hop_parent_sort: Rc<Cell<Option<Sort>>>, // sort of the hop parent, or none if not in hop
}

#[glib::object_subclass]
impl ObjectSubclass for MViewWindowImp {
    const NAME: &'static str = "MViewWindow";
    type Type = super::MViewWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl MViewWindowImp {
    pub fn show_files_widget(&self, show: bool) {
        let w = self.widgets.get().unwrap();
        if w.files_widget.is_visible() != show {
            w.files_widget.set_visible(show);
            if show {
                w.hbox.set_spacing(8);
                self.obj().set_border_width(10);
            } else {
                w.hbox.set_spacing(0);
                self.obj().set_border_width(0);
            }

        }
    }
}

impl ObjectImpl for MViewWindowImp {
    fn constructed(&self) {
        self.parent_constructed();
        self.full_screen.set(false);
        self.skip_loading.set(false);
        self.current_sort.set(Some(Sort::default()));

        let window = self.obj();

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

        let file_list_view = FileListView::new();
        file_list_view.set_vexpand(true);
        files_widget.add(&file_list_view);

        let eog = ScrollView::new();
        eog.add_weak_ref_notify(|| {
            println!("**eog::ScrollView disposed**");
        });
        eog.set_scroll_wheel_zoom(true);
        eog.set_zoom_mode(eog::ZoomMode::Fill);
        hbox.add(&eog);

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
                backend: RefCell::new(<dyn Backend>::invalid()),
                file_list_view,
                files_widget,
                eog,
            })
            .expect("Failed to initialize MView window");

        window.show_all();

        self.set_backend(<dyn Backend>::current_dir(), None);

        // self.widgets.get().unwrap().eog.set_offset(0, 0);

        println!("MViewWindowSub: constructed done");
    }
}

impl WidgetImpl for MViewWindowImp {}
impl ContainerImpl for MViewWindowImp {}
impl BinImpl for MViewWindowImp {}
impl WindowImpl for MViewWindowImp {}
impl ApplicationWindowImpl for MViewWindowImp {}
