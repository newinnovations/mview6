mod category;
mod filelist;
pub mod filelist_view;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use filelist::FileList;
use filelist::Navigation;
use filelist::TreeviewExtManual;
use filelist_view::FileListView;

use gdk::glib::ObjectExt;
use gdk::Screen;
use gtk::glib;

use eog::Image;
use eog::ImageData;
use eog::ImageExtManual;
use eog::Job;
use eog::ScrollView;

use eog::prelude::ImageExt;
use eog::prelude::ScrollViewExt;

use gtk::prelude::ApplicationExt;
use gtk::prelude::ApplicationExtManual;
use gtk::prelude::BoxExt;
use gtk::prelude::ContainerExt;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::ScrolledWindowExt;
use gtk::prelude::TreeViewExt;
use gtk::prelude::WidgetExt;
use gtk::StyleContext;

use gtk::prelude::CssProviderExt;
use gtk::CssProvider;
use gtk::STYLE_PROVIDER_PRIORITY_USER;

fn main() {
    let application = gtk::Application::new(Some("org.vanderwerff.mview.gtk3"), Default::default());

    application.connect_startup(build_ui);

    application.connect_activate(|_| {
        println!("connect_activate");
    });

    application.run();
}

fn build_ui(application: &gtk::Application) {

    let window = gtk::ApplicationWindow::new(application);
    window.set_title("MView6");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

    let css_provider = CssProvider::new();
    css_provider
        .load_from_data(include_bytes!("mview6.css"))
        .unwrap();
    StyleContext::add_provider_for_screen(
        &Screen::default().unwrap(),
        &css_provider,
        STYLE_PROVIDER_PRIORITY_USER,
    );

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    window.add(&hbox);

    let file_window = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    file_window.set_shadow_type(gtk::ShadowType::EtchedIn);
    file_window.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    hbox.add(&file_window);

    // let model = Rc::new(FileList::create_model());
    // let treeview = gtk::TreeView::with_model(&*model);
    // let filelist = Rc<FileList>::;
    let filelist = Rc::new(RefCell::new(FileList::new("/home/martin/Pictures")));
    // let treeview = gtk::TreeView::new();

    let treeview = FileListView::new();
    // let treeview = treeviewx.upcast::<TreeView>();

    treeview.set_model(filelist.borrow().read().as_ref());
    treeview.set_vexpand(true);
    // treeview.set_search_column(Columns::Name as i32);

    file_window.add(&treeview);

    // add_columns(&treeview);

    let sv = ScrollView::new();
    sv.add_weak_ref_notify(|| {
        println!("ScrollView disposed");
    });
    sv.set_scroll_wheel_zoom(true);

    hbox.add(&sv);

    sv.set_zoom_mode(eog::ZoomMode::Max);

    let sv_c = sv.clone();
    let f_c = filelist.clone();
    treeview.connect_cursor_changed(move |tv| {
        if let Some(filename) = Navigation::filename(&tv) {
            println!("Selected file {}", filename);
            let path = format!("{0}/{filename}", f_c.borrow().directory);
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
                    sv_c.set_image(&img);
                }
                Err(error) => {
                    println!("Error {}", error);
                }
            }
        }
    });

    let fs = Cell::new(false);
    let treeview_c = treeview.clone();
    let sv_c = sv.clone();
    let f_c = filelist.clone();
    window.connect_key_press_event(move |app, e| {
        // println!("Key {}", e.keycode().unwrap());
        treeview_c.set_has_focus(true);
        match e.keyval() {
            gdk::keys::constants::q => {
                app.close();
            }
            gdk::keys::constants::space => {
                if file_window.is_visible() {
                    file_window.set_visible(false);
                    hbox.set_spacing(0);
                    app.set_border_width(0);
                } else {
                    file_window.set_visible(true);
                    hbox.set_spacing(8);
                    app.set_border_width(10);
                }
            }
            gdk::keys::constants::Return => {
                if let Some(subdir) = Navigation::filename(&treeview_c) {
                    let mut filelist = f_c.borrow_mut();
                    let newstore = filelist.enter(&subdir);
                    drop(filelist);
                    if newstore.is_some() {
                        treeview_c.set_model(newstore.as_ref());
                        treeview_c.goto_first();
                    }
                }
            }
            gdk::keys::constants::BackSpace => {
                let mut filelist = f_c.borrow_mut();
                let newstore = filelist.leave();
                drop(filelist);
                treeview_c.set_model(newstore.as_ref());
                treeview_c.goto_first();
            }
            gdk::keys::constants::d => {
                Navigation::write(&treeview_c);
            }
            gdk::keys::constants::f => {
                if fs.get() {
                    app.unfullscreen();
                    fs.set(false);
                } else {
                    file_window.set_visible(false);
                    hbox.set_spacing(0);
                    app.set_border_width(0);
                    app.fullscreen();
                    fs.set(true);
                }
            }
            gdk::keys::constants::o => {
                if sv_c.zoom_mode() == eog::ZoomMode::Fit {
                    sv_c.set_zoom_mode(eog::ZoomMode::None);
                } else {
                    sv_c.set_zoom_mode(eog::ZoomMode::Fit);
                }
            }
            gdk::keys::constants::m => {
                if sv_c.zoom_mode() == eog::ZoomMode::Max {
                    sv_c.set_zoom_mode(eog::ZoomMode::Fill);
                } else {
                    sv_c.set_zoom_mode(eog::ZoomMode::Max);
                }
            }
            gdk::keys::constants::z | gdk::keys::constants::Left => {
                treeview_c.emit_move_cursor(gtk::MovementStep::DisplayLines, -1);
            }
            gdk::keys::constants::x | gdk::keys::constants::Right => {
                treeview_c.emit_move_cursor(gtk::MovementStep::DisplayLines, 1);
            }
            gdk::keys::constants::Page_Up => {
                treeview_c.emit_move_cursor(gtk::MovementStep::Pages, -1);
            }
            gdk::keys::constants::Page_Down => {
                treeview_c.emit_move_cursor(gtk::MovementStep::Pages, 1);
            }
            gdk::keys::constants::Home => {
                treeview_c.emit_move_cursor(gtk::MovementStep::BufferEnds, -1);
            }
            gdk::keys::constants::End => {
                treeview_c.emit_move_cursor(gtk::MovementStep::BufferEnds, 1);
            }
            gdk::keys::constants::Up => {
                let (tp, col) = treeview_c.cursor();
                if let Some(mut tp) = tp {
                    println!("tp: {:?}", tp.indices());
                    // TreePath::from_indicesv(&[3]);
                    // let n = tp.indices().get(0).unwrap().to_owned();
                    // let m = treeview_c.model().unwrap();
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
                    treeview_c.set_cursor(&tp, col.as_ref(), false);
                }
            }
            gdk::keys::constants::Down => {
                let (tp, col) = treeview_c.cursor();
                if let Some(mut tp) = tp {
                    println!("tp: {:?}", tp.indices());
                    for _ in 0..1 {
                        tp.next();
                    }
                    treeview_c.set_cursor(&tp, col.as_ref(), false);
                }
            }
            _ => (),
        }
        glib::Propagation::Stop
    });

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

    window.show_all();
}
