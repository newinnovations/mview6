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
    image::view::{ImageView, ZoomMode},
    widget::MViewWidgetExt,
};
use gdk_pixbuf::PixbufLoader;
use glib::{clone, once_cell::unsync::OnceCell};
use gtk::{
    glib::{self, Propagation},
    prelude::*,
    subclass::prelude::*,
    ScrolledWindow,
};
use std::cell::{Cell, RefCell};

#[derive(Debug)]
pub struct MViewWidgets {
    hbox: gtk::Box,
    files_widget: ScrolledWindow,
    file_list_view: FileListView,
    image_view: ImageView,
    pub sender: glib::Sender<Message>,
}

#[derive(Debug, Default)]
pub struct MViewWindowImp {
    widget_cell: OnceCell<MViewWidgets>,
    backend: RefCell<Box<dyn Backend>>,
    full_screen: Cell<bool>,
    skip_loading: Cell<bool>,
    thumbnail_size: Cell<i32>,
    current_sort: Cell<Sort>,
}

#[glib::object_subclass]
impl ObjectSubclass for MViewWindowImp {
    const NAME: &'static str = "MViewWindow";
    type Type = super::MViewWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl MViewWindowImp {
    fn widgets(&self) -> &MViewWidgets {
        self.widget_cell.get().unwrap()
    }

    pub fn show_files_widget(&self, show: bool) {
        let w = self.widgets();
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

        let image_view = ImageView::new();
        image_view.set_zoom_mode(ZoomMode::Fill);
        hbox.add(&image_view);

        window.connect_key_press_event(
            clone!(@weak self as this => @default-return Propagation::Stop, move |_, e| {
                this.on_key_press(e);
                Propagation::Stop
            }),
        );

        image_view.connect_motion_notify_event(
            clone!(@weak self as this => @default-return Propagation::Stop, move |_, e| {
                this.on_mouse_move(e);
                Propagation::Proceed
            }),
        );

        image_view.connect_button_press_event(
            clone!(@weak self as this => @default-return Propagation::Stop, move |_, e| {
                this.on_mouse_press(e);
                Propagation::Proceed
            }),
        );

        file_list_view.connect_cursor_changed(clone!(@weak self as this => move |_| {
            this.on_cursor_changed();
        }));

        file_list_view.connect_row_activated(clone!(@weak self as this => move |_, path, column| {
            this.on_row_activated(path, column);
        }));

        // TODO: refactor to https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html
        #[allow(deprecated)]
        let (sender, receiver) = glib::MainContext::channel::<Message>(glib::Priority::DEFAULT);

        self.widget_cell
            .set(MViewWidgets {
                hbox,
                file_list_view,
                files_widget,
                image_view,
                sender,
            })
            .expect("Failed to initialize MView window");

        let w = self.widgets();
        let image_view = w.image_view.clone();
        let sender = w.sender.clone();
        let mut current_task = 0;
        let mut command = TCommand::default();
        receiver.attach(None, move |msg| {
            match msg {
                Message::Command(cmd) => {
                    command = cmd;
                    current_task = 0;
                    if command.needs_work() {
                        start_thumbnail_task(&sender, &image_view, &command, &mut current_task);
                        start_thumbnail_task(&sender, &image_view, &command, &mut current_task);
                        start_thumbnail_task(&sender, &image_view, &command, &mut current_task);
                    } else {
                        image_view.set_image_post();
                    }
                }
                Message::Result(res) => {
                    if handle_thumbnail_result(&image_view, &mut command, res) {
                        start_thumbnail_task(&sender, &image_view, &command, &mut current_task);
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        window.show_all();

        let display_size = window.display_size();
        dbg!(display_size);

        self.set_backend(<dyn Backend>::current_dir(), Selection::None, false);

        println!("MViewWindowSub: constructed done");
    }
}

impl WidgetImpl for MViewWindowImp {}
impl ContainerImpl for MViewWindowImp {}
impl BinImpl for MViewWindowImp {}
impl WindowImpl for MViewWindowImp {}
impl ApplicationWindowImpl for MViewWindowImp {}
