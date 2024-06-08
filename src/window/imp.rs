use eog::{Image, ImageData, ImageExt, ImageExtManual, Job, ScrollView, ScrollViewExt};
use gdk::EventKey;
use glib::{clone, once_cell::unsync::OnceCell};
use gtk::{glib, prelude::*, subclass::prelude::*, Box, ScrolledWindow};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::filelist::FileList;
use crate::filelistview::FileListView;
use crate::filelistview::FileListViewExt;

#[derive(Debug)]
struct MViewWidgets {
    hbox: Box,
    file_list: Rc<RefCell<FileList>>,
    file_window: ScrolledWindow,
    sv: ScrollView,
    treeview: FileListView,
}

#[derive(Debug, Default)]
pub struct MViewWindowSub {
    widgets: OnceCell<MViewWidgets>,
    fs: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for MViewWindowSub {
    const NAME: &'static str = "MViewWindow";
    type Type = super::MViewWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for MViewWindowSub {
    // Here we are overriding the glib::Objcet::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
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

        let f = gio::File::for_path("/home/martin/Pictures/mview-a.png");
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
            imp.on_key_press(&e);
            glib::Propagation::Stop
        }));

        treeview.connect_cursor_changed(clone!(@weak self as imp => move |_| {
            imp.on_cursor_changed();
        }));


        // let headerbar = gtk::HeaderBar::new();
        // let increment = gtk::Button::with_label("Increment!");
        // let label = gtk::Label::new(Some("Press the Increment Button!"));

        // headerbar.set_title(Some("Hello World!"));
        // headerbar.set_show_close_button(true);
        // headerbar.pack_start(&increment);

        // // Connect our method `on_increment_clicked` to be called
        // // when the increment button is clicked.
        // increment.connect_clicked(clone!(@weak self as imp => move |_| {
        //     imp.on_increment_clicked();
        // }));

        // instance.add(&label);
        // instance.set_titlebar(Some(&headerbar));
        // instance.set_default_size(640, 480);

        self.widgets
            .set(MViewWidgets {
                file_list,
                file_window,
                hbox,
                sv,
                treeview,
            })
            .expect("Failed to initialize MView window");
    }
}

impl MViewWindowSub {
    fn on_key_press(&self, e: &EventKey) {
        println!("Key {}", e.keycode().unwrap());
        let w = self.widgets.get().unwrap();
        w.treeview.set_has_focus(true);
        match e.keyval() {
            gdk::keys::constants::q => {
                self.obj().close();
            }
            gdk::keys::constants::space => {
                if w.file_window.is_visible() {
                    w.file_window.set_visible(false);
                    w.hbox.set_spacing(0);
                    self.obj().set_border_width(0);
                } else {
                    w.file_window.set_visible(true);
                    w.hbox.set_spacing(8);
                    self.obj().set_border_width(10);
                }
            }
            gdk::keys::constants::f => {
                if self.fs.get() {
                    self.obj().unfullscreen();
                    self.fs.set(false);
                } else {
                    w.file_window.set_visible(false);
                    w.hbox.set_spacing(0);
                    self.obj().set_border_width(0);
                    self.obj().fullscreen();
                    self.fs.set(true);
                }
            }
            gdk::keys::constants::Return => {
                if let Some(subdir) = &w.treeview.filename() {
                    let mut filelist = w.file_list.borrow_mut();
                    let newstore = filelist.enter(&subdir);
                    drop(filelist);
                    if newstore.is_some() {
                        w.treeview.set_model(newstore.as_ref());
                        w.treeview.goto_first();
                    }
                }
            }
            gdk::keys::constants::BackSpace => {
                let mut filelist = w.file_list.borrow_mut();
                let newstore = filelist.leave();
                drop(filelist);
                w.treeview.set_model(newstore.as_ref());
                w.treeview.goto_first();
            }
            gdk::keys::constants::d => {
                w.treeview.write();
            }
            gdk::keys::constants::o => {
                if w.sv.zoom_mode() == eog::ZoomMode::Fit {
                    w.sv.set_zoom_mode(eog::ZoomMode::None);
                } else {
                    w.sv.set_zoom_mode(eog::ZoomMode::Fit);
                }
            }
            gdk::keys::constants::m => {
                if w.sv.zoom_mode() == eog::ZoomMode::Max {
                    w.sv.set_zoom_mode(eog::ZoomMode::Fill);
                } else {
                    w.sv.set_zoom_mode(eog::ZoomMode::Max);
                }
            }
            gdk::keys::constants::z | gdk::keys::constants::Left => {
                w.treeview.emit_move_cursor(gtk::MovementStep::DisplayLines, -1);
            }
            gdk::keys::constants::x | gdk::keys::constants::Right => {
                w.treeview.emit_move_cursor(gtk::MovementStep::DisplayLines, 1);
            }
            gdk::keys::constants::Page_Up => {
                w.treeview.emit_move_cursor(gtk::MovementStep::Pages, -1);
            }
            gdk::keys::constants::Page_Down => {
                w.treeview.emit_move_cursor(gtk::MovementStep::Pages, 1);
            }
            gdk::keys::constants::Home => {
                w.treeview.emit_move_cursor(gtk::MovementStep::BufferEnds, -1);
            }
            gdk::keys::constants::End => {
                w.treeview.emit_move_cursor(gtk::MovementStep::BufferEnds, 1);
            }
            gdk::keys::constants::Up => {
                let (tp, col) = w.treeview.cursor();
                if let Some(mut tp) = tp {
                    println!("tp: {:?}", tp.indices());
                    // TreePath::from_indicesv(&[3]);
                    // let n = tp.indices().get(0).unwrap().to_owned();
                    // let m = w.treeview.model().unwrap();
                    // let i = m.iter_nth_child(None, n).unwrap();
                    // println!(
                    //     "Current = {}",
                    //     model
                    //         .value(&i, Columns::Name as i32)
                    //         .get::<String>()
                    //         .unwrap_or("??".to_string())
                    // );
                    for _ in 0..1 {
                        tp.prev();
                    }
                    w.treeview.set_cursor(&tp, col.as_ref(), false);
                }
            }
            gdk::keys::constants::Down => {
                let (tp, col) = w.treeview.cursor();
                if let Some(mut tp) = tp {
                    println!("tp: {:?}", tp.indices());
                    for _ in 0..1 {
                        tp.next();
                    }
                    w.treeview.set_cursor(&tp, col.as_ref(), false);
                }
            }
            _ => (),
        }
    }

    fn on_cursor_changed(&self) {
        let w = self.widgets.get().unwrap();
        if let Some(filename) = w.treeview.filename() {
            println!("Selected file {}", filename);
            let path = format!("{0}/{filename}", w.file_list.borrow().directory);
            println!("Path = {}", path);
            let f = gio::File::for_path(path);
            let img = Image::new_file(&f, "blah");
            img.add_weak_ref_notify(move || {
                println!("**image [{}] disposed**", filename);
            });
            // println!("refc1={}", img.ref_count());
            // // img.data_ref();
            // // img.data_unref();
            // println!("refc2={}", img.ref_count());
            let result = img.load(ImageData::IMAGE, None::<Job>.as_ref());
            match result {
                Ok(()) => {
                    let (width, height) = img.size();
                    println!("OK: size {} {}", width, height);
                    w.sv.set_image(&img);
                }
                Err(error) => {
                    println!("Error {}", error);
                }
            }
        }

    }
}

impl WidgetImpl for MViewWindowSub {}
impl ContainerImpl for MViewWindowSub {}
impl BinImpl for MViewWindowSub {}
impl WindowImpl for MViewWindowSub {}
impl ApplicationWindowImpl for MViewWindowSub {}
