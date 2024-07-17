mod backend;
mod cursor;
mod keyboard;

use crate::{
    backends::{filesystem::FileSystem, Backend, Columns},
    filelistview::FileListView,
    loader::Loader,
    window::{Message, TCommand, TResult},
};
use eog::{ImageExt, ScrollView, ScrollViewExt};
use gdk::{Display, Rectangle};
use gdk_pixbuf::PixbufLoader;
use glib::{clone, once_cell::unsync::OnceCell};
use gtk::{glib, prelude::*, subclass::prelude::*, ScrolledWindow, SortColumn, SortType};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    thread,
};

use super::{MViewWidgets, TSource};

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

        #[allow(deprecated)]
        let (sender, receiver) = glib::MainContext::channel::<Message>(glib::Priority::DEFAULT);

        self.widgets
            .set(MViewWidgets {
                hbox,
                backend: RefCell::new(<dyn Backend>::invalid()),
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
                Message::UpdateLabel(text) => {
                    println!("Command {:?}", command);
                    println!("Test {current_task}");
                    println!("Text {text}");
                    // let image = draw(&text).unwrap();
                    // eog.set_image(&image);
                }
                Message::Image(im) => {
                    println!("Command {:?}", command);
                    println!("Image {current_task}");
                    if let Ok(pb) = Loader::image_rs_to_pixbuf(im) {
                        if let Some(i) = eog.image() {
                            println!("im id = {}", i.id());
                            if let Some(p) = i.pixbuf() {
                                pb.copy_area(
                                    0,
                                    0,
                                    pb.width(),
                                    pb.height(),
                                    &p,
                                    p.width() / 2,
                                    p.height() / 2,
                                );
                                i.modified();
                            }
                        }
                    }
                }
                Message::Command(cmd) => {
                    command = cmd;
                    current_task = 0;
                    start_thumbnail_task(&sender, &eog, &command, &mut current_task);
                    start_thumbnail_task(&sender, &eog, &command, &mut current_task);
                }
                Message::Result(res) => {
                    if handle_thumbnail_result(&eog, res) {
                        start_thumbnail_task(&sender, &eog, &command, &mut current_task);
                    }
                } // Message::Test(tst) => {
                  //     println!("Command {:?}", command);
                  //     println!("Test {tst}");
                  //     current_task = tst;
                  // }
            }

            // Returning false here would close the receiver
            // and have senders fail
            glib::ControlFlow::Continue
        });

        window.show_all();

        let display_size = window.display_size();
        dbg!(display_size);

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

pub trait MViewWidgetExt: IsA<gtk::Widget> {
    fn display_size(&self) -> gdk::Rectangle;
}

impl<O: IsA<gtk::Widget>> MViewWidgetExt for O {
    fn display_size(&self) -> gdk::Rectangle {
        if let Some(display) = Display::default() {
            if let Some(window) = self.window() {
                if let Some(monitor) = display.monitor_at_window(&window) {
                    return monitor.workarea();
                }
            }
        }
        Rectangle::new(0, 0, 800, 600)
    }
}

fn start_thumbnail_task(
    sender: &glib::Sender<Message>,
    eog: &ScrollView,
    command: &TCommand,
    current_task: &mut usize,
) {
    if let Some(image) = eog.image() {
        let id = image.id();
        if command.id == id {
            println!("Command id is ok: {id}");
            let sender_clone = sender.clone();
            if let Some(task) = command.tasks.get(*current_task) {
                *current_task += 1;
                let task = task.clone();
                thread::spawn(move || {
                    // thread::sleep(time::Duration::from_secs(2));
                    let image = match &task.source {
                        TSource::FileSource(src) => FileSystem::get_thumbnail(src),
                        TSource::None => None,
                    };

                    let image = match image {
                        Some(im) => Some(im.resize(
                            task.size,
                            task.size,
                            image::imageops::FilterType::Lanczos3,
                        )),
                        None => None,
                    };

                    let _ = sender_clone.send(Message::Result(TResult::new(id, task, image)));
                });
            }
        } else {
            println!("Command id mismatch {} != {id}", command.id);
        }
    }
}

fn handle_thumbnail_result(eog: &ScrollView, res: TResult) -> bool {
    if let Some(image) = eog.image() {
        let id = image.id();
        if res.id == id {
            println!("Result id is ok: {id}");
            if let Some(thumb) = res.image {
                println!("-- got thumb image");
                if let Ok(thumb_pb) = Loader::image_rs_to_pixbuf(thumb) {
                    if let Some(image_pb) = image.pixbuf() {
                        let size = res.task.size as i32;
                        let (x, y) = res.task.position;
                        thumb_pb.copy_area(
                            0,
                            0,
                            thumb_pb.width(),
                            thumb_pb.height(),
                            &image_pb,
                            x + (size - thumb_pb.width()) / 2,
                            y + (size - thumb_pb.height()) / 2,
                        );
                        image.modified();
                    }
                }
            } else {
                println!("-- no thumb image");
            }
            return true;
        } else {
            println!("Command id mismatch {} != {id}", res.id);
        }
    }
    false
}
