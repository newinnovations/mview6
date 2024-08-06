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
    filelistview::{FileListView, Filter, Selection, Sort},
    image::view::{ImageView, ZoomMode, SIGNAL_VIEW_RESIZED},
};
use async_channel::Sender;
use glib::{clone, closure_local};
use gtk4::{
    glib::{self, Propagation},
    prelude::*,
    subclass::prelude::*,
    EventControllerKey, ScrolledWindow,
};
use std::cell::{Cell, OnceCell, RefCell};

#[derive(Debug)]
pub struct MViewWidgets {
    hbox: gtk4::Box,
    files_widget: ScrolledWindow,
    file_list_view: FileListView,
    image_view: ImageView,
    pub sender: Sender<Message>,
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
    type ParentType = gtk4::ApplicationWindow;
}

impl MViewWindowImp {
    fn widgets(&self) -> &MViewWidgets {
        self.widget_cell.get().unwrap()
    }

    pub fn show_files_widget(&self, show: bool, force: bool) {
        let w = self.widgets();
        if w.files_widget.is_visible() != show || force {
            w.files_widget.set_visible(show);
            let border = if show { 8 } else { 0 };
            w.hbox.set_spacing(border);
            w.file_list_view.set_margin_start(border);
            w.file_list_view.set_margin_top(border);
            w.file_list_view.set_margin_bottom(border);
            w.image_view.set_margin_end(border);
            w.image_view.set_margin_top(border);
            w.image_view.set_margin_bottom(border);
        }
    }
}

impl ObjectImpl for MViewWindowImp {
    fn constructed(&self) {
        self.parent_constructed();
        self.thumbnail_size.set(175);
        self.current_sort.set(Sort::sort_on_category());

        let window = self.obj();

        window.set_title(Some("MView6"));
        // window.set_position(gtk4::WindowPosition::Center); TODO
        window.set_default_size(1280, 720);

        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let files_widget = ScrolledWindow::new();
        // files_widget.set_shadow_type(gtk4::ShadowType::EtchedIn); TODO
        files_widget.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        files_widget.set_can_focus(false);
        hbox.append(&files_widget);

        let file_list_view = FileListView::new();
        file_list_view.set_vexpand(true);
        file_list_view.set_fixed_height_mode(true);
        file_list_view.set_can_focus(false);
        files_widget.set_child(Some(&file_list_view));

        let image_view = ImageView::new();
        image_view.set_zoom_mode(ZoomMode::Fill);
        hbox.append(&image_view);

        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            Propagation::Stop,
            move |_ctrl, key, _, _| {
                this.on_key_press(key);
                Propagation::Stop
            }
        ));
        self.obj().add_controller(key_controller);

        let gesture_click = gtk4::GestureClick::new();
        gesture_click.set_button(1);
        gesture_click.connect_pressed(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, _n_press, x, y| this.on_mouse_press((x, y))
        ));
        image_view.add_controller(gesture_click);

        image_view.connect_closure(
            SIGNAL_VIEW_RESIZED,
            false,
            closure_local!(
                #[weak(rename_to = this)]
                self,
                move |_view: ImageView, width: i32, height: i32| {
                    println!("view was resized to {width} {height}");
                    this.update_thumbnail_backend();
                }
            ),
        );

        file_list_view.connect_cursor_changed(clone!(
            #[weak(rename_to = this)]
            self,
            move |_| this.on_cursor_changed()
        ));

        file_list_view.connect_row_activated(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, path, column| {
                this.on_row_activated(path, column);
            }
        ));

        let (sender, receiver) = async_channel::unbounded::<Message>();

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
        glib::spawn_future_local(clone!(
            #[strong(rename_to = image_view)]
            w.image_view,
            #[strong(rename_to = sender)]
            w.sender,
            async move {
                let mut current_task = 0;
                let mut command = TCommand::default();
                while let Ok(msg) = receiver.recv().await {
                    match msg {
                        Message::Command(cmd) => {
                            command = cmd;
                            current_task = 0;
                            if command.needs_work() {
                                start_thumbnail_task(
                                    &sender,
                                    &image_view,
                                    &command,
                                    &mut current_task,
                                );
                                start_thumbnail_task(
                                    &sender,
                                    &image_view,
                                    &command,
                                    &mut current_task,
                                );
                                start_thumbnail_task(
                                    &sender,
                                    &image_view,
                                    &command,
                                    &mut current_task,
                                );
                            } else {
                                image_view.set_image_post();
                            }
                        }
                        Message::Result(res) => {
                            if handle_thumbnail_result(&image_view, &mut command, res) {
                                start_thumbnail_task(
                                    &sender,
                                    &image_view,
                                    &command,
                                    &mut current_task,
                                );
                            }
                        }
                    }
                }
            }
        ));

        self.show_files_widget(true, true);
        window.set_child(Some(&w.hbox));
        window.show();

        self.set_backend(<dyn Backend>::current_dir(), Selection::None, false);

        println!("MViewWindow: constructed");
    }
}

impl WidgetImpl for MViewWindowImp {}
impl WindowImpl for MViewWindowImp {}
impl ApplicationWindowImpl for MViewWindowImp {}

impl MViewWidgets {
    pub fn filter(&self) -> Filter {
        if self.files_widget.is_visible() {
            Filter::None
        } else {
            Filter::Image
        }
    }
}
