mod backend;
mod keyboard;
mod mouse;
mod navigate;

use crate::{
    backends::{
        thumbnail::{
            processing::{handle_thumbnail_result, start_thumbnail_task},
            Message, TCommand,
        },
        Backend,
    },
    filelistview::{FileListView, Selection, Sort},
    image_view::{ImageView, ZoomMode},
    widget::MViewWidgetExt,
};
use gdk_pixbuf::PixbufLoader;
use glib::{clone, once_cell::unsync::OnceCell};
use gtk::{glib, prelude::*, subclass::prelude::*, ScrolledWindow};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::MViewWidgets;

#[derive(Debug, Default)]
pub struct MViewWindowImp {
    widgets: OnceCell<MViewWidgets>,
    full_screen: Cell<bool>,
    skip_loading: Cell<bool>,
    thumbnail_size: Cell<i32>,
    current_sort: Rc<Cell<Sort>>,
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
        self.thumbnail_size.set(175);
        self.current_sort.set(Sort::sort_on_category());

        let window = self.obj();

        window.set_title("MView6");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(1280, 720);

        let loader = PixbufLoader::with_type("svg").unwrap();
        loader
            .write(include_bytes!("../../resources/icon.svg"))
            .unwrap();
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
        file_list_view.set_fixed_height_mode(true);
        files_widget.add(&file_list_view);

        let eog = ImageView::new();
        eog.add_weak_ref_notify(|| {
            println!("**eog::ScrollView disposed**");
        });
        eog.set_scroll_wheel_zoom(true);
        eog.set_zoom_mode(ZoomMode::Fill);
        hbox.add(&eog);

        window.connect_key_press_event(clone!(@weak self as imp => @default-panic, move |_, e| {
            imp.on_key_press(e);
            glib::Propagation::Stop
        }));

        eog.connect_motion_notify_event(clone!(@weak self as imp => @default-panic, move |_, e| {
            imp.on_mouse_move(e);
            glib::Propagation::Proceed
        }));

        eog.connect_button_press_event(clone!(@weak self as imp => @default-panic, move |_, e| {
            imp.on_mouse_press(e);
            glib::Propagation::Proceed
        }));

        file_list_view.connect_cursor_changed(clone!(@weak self as imp => move |_| {
            imp.on_cursor_changed();
        }));

        file_list_view.connect_row_activated(clone!(@weak self as imp => move |_, path, column| {
            imp.on_row_activated(path, column);
        }));

        #[allow(deprecated)]
        let (sender, receiver) = glib::MainContext::channel::<Message>(glib::Priority::DEFAULT);

        self.widgets
            .set(MViewWidgets {
                hbox,
                backend: RefCell::new(<dyn Backend>::none()),
                file_list_view,
                files_widget,
                eog,
                sender,
            })
            .expect("Failed to initialize MView window");

        let w = self.widgets.get().unwrap();
        let eog = w.eog.clone();
        let sender = w.sender.clone();
        let mut current_task = 0;
        let mut command = TCommand::default();
        receiver.attach(None, move |msg| {
            match msg {
                Message::Command(cmd) => {
                    command = cmd;
                    current_task = 0;
                    if command.needs_work() {
                        start_thumbnail_task(&sender, &eog, &command, &mut current_task);
                        start_thumbnail_task(&sender, &eog, &command, &mut current_task);
                        start_thumbnail_task(&sender, &eog, &command, &mut current_task);
                    } else {
                        eog.set_image_post();
                    }
                }
                Message::Result(res) => {
                    if handle_thumbnail_result(&eog, &mut command, res) {
                        start_thumbnail_task(&sender, &eog, &command, &mut current_task);
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        window.show_all();

        let display_size = window.display_size();
        dbg!(display_size);

        self.set_backend(<dyn Backend>::current_dir(), Selection::None, false);

        // self.widgets.get().unwrap().eog.set_offset(0, 0);

        println!("MViewWindowSub: constructed done");
    }
}

impl WidgetImpl for MViewWindowImp {}
impl ContainerImpl for MViewWindowImp {}
impl BinImpl for MViewWindowImp {}
impl WindowImpl for MViewWindowImp {}
impl ApplicationWindowImpl for MViewWindowImp {}
