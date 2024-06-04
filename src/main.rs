use std::cell::Cell;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::rc::Rc;
use std::time::UNIX_EPOCH;

use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;

use gdk::glib::ObjectExt;
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
use gtk::prelude::GtkListStoreExtManual;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::ScrolledWindowExt;
use gtk::prelude::TreeModelExt;
use gtk::prelude::TreeSelectionExt;
use gtk::prelude::TreeViewColumnExt;
use gtk::prelude::TreeViewExt;
use gtk::prelude::WidgetExt;

// use gtk::prelude::CssProviderExt;
// use gtk::prelude::StyleContextExt;
// use gtk::CssProvider;

fn main() {
    let application = gtk::Application::new(Some("org.vanderwerff.mview.gtk3"), Default::default());

    application.connect_startup(build_ui);

    application.connect_activate(|_| {
        println!("connect_activate");
    });

    application.run();
}

#[derive(Debug)]
#[repr(i32)]
enum Columns {
    Cat = 0,
    Name,
    Size,
    Modified,
}

fn read_directory(store: &gtk::ListStore, current_dir: &str) -> io::Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        let filename = path.file_name().unwrap_or(OsStr::new("-"));
        let filename = filename.to_str().unwrap_or("-");
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        let modified = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let file_size = metadata.len();
        let cat: u32 = if metadata.is_dir() { 1 } else { 0 };
        store.insert_with_values(
            None,
            &[(0, &cat), (1, &filename), (2, &file_size), (3, &modified)],
        );
    }
    Ok(())
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("List Store");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);
    // let sc = window.style_context();
    // let style = include_bytes!("box.css");
    // let provider = CssProvider::new();
    // provider.load_from_data(style).unwrap();
    // sc.add_provider(&provider, GTK_STYLE_PROVIDER_PRIORITY_APPLICATION as u32);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    window.add(&hbox);

    let file_window = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    file_window.set_shadow_type(gtk::ShadowType::EtchedIn);
    file_window.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    hbox.add(&file_window);

    let model = Rc::new(create_model());
    let treeview = gtk::TreeView::with_model(&*model);
    treeview.set_vexpand(true);
    // treeview.set_search_column(Columns::Name as i32);

    file_window.add(&treeview);

    add_columns(&treeview);

    let sv = ScrollView::new();
    sv.add_weak_ref_notify(|| {
        println!("ScrollView disposed");
    });
    sv.set_scroll_wheel_zoom(true);

    hbox.add(&sv);

    sv.set_zoom_mode(eog::ZoomMode::Max);

    let sv_c = sv.clone();
    treeview.connect_cursor_changed(move |tv| {
        // let w = tv.parent_window().unwrap().s::<ApplicationWindow>();
        // println!("TV parent window = {}", tv.parent_window().unwrap().type_());
        let selection = tv.selection();
        if let Some((model, iter)) = selection.selected() {
            let filename = model
                .value(&iter, 1)
                .get::<String>()
                .unwrap_or("none".to_string());
            println!("Selected file {}", filename);
            let mut path = "/home/martin/Pictures/".to_string();
            path.push_str(&filename);
            println!("Path = {}", path);
            let f = gio::File::for_path(path);
            let img = Image::new_file(&f, "blah");
            img.add_weak_ref_notify(move || {
                println!("**image [{}] disposed**", filename);
            });
            println!("refc1={}", img.ref_count());
            // img.data_ref();
            // img.data_unref();
            println!("refc2={}", img.ref_count());
            let result = img.load(ImageData::IMAGE, None::<Job>.as_ref());
            match result {
                Ok(()) => {
                    println!("OK");
                    let jpg = img.is_jpeg();
                    println!("is jpg {}", jpg);
                    let mut width = 0;
                    let mut height = 0;
                    img.size(&mut width, &mut height);
                    println!("Size {} {}", width, height);
                    sv_c.set_image(&img);
                    // sv_c.apply_zoom(eog::ZoomMode::Max);
                    // sv_c.apply_zoom(eog::ZoomMode::None);
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
    window.connect_key_press_event(move |app, e| {
        println!("Key {}", e.keycode().unwrap());
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
                    for _ in 0..5 {
                        tp.prev();
                    }
                    treeview_c.set_cursor(&tp, col.as_ref(), false);
                }
            }
            gdk::keys::constants::Down => {
                let (tp, col) = treeview_c.cursor();
                if let Some(mut tp) = tp {
                    for _ in 0..5 {
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

            let mut width = 0;
            let mut height = 0;
            img.size(&mut width, &mut height);

            println!("Size {} {}", width, height);

            sv.set_image(&img);
        }
        Err(error) => {
            println!("Error {}", error);
        }
    }

    window.show_all();
}

fn create_model() -> gtk::ListStore {
    let col_types: [glib::Type; 5] = [
        glib::Type::U32,
        glib::Type::STRING,
        glib::Type::U64,
        glib::Type::U64,
        glib::Type::STRING,
    ];
    let store = gtk::ListStore::new(&col_types);
    let current_dir = "/home/martin/Pictures";
    let _ = read_directory(&store, &current_dir);
    store
}

fn add_columns(treeview: &gtk::TreeView) {
    // Column for category
    let renderer = gtk::CellRendererText::new();
    let column = gtk::TreeViewColumn::new();
    column.pack_start(&renderer, true);
    column.set_title("Cat");
    column.add_attribute(&renderer, "text", Columns::Cat as i32);
    column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
    column.set_fixed_width(40);
    column.set_sort_column_id(Columns::Cat as i32);
    treeview.append_column(&column);

    // Column for file/direcory
    let renderer = gtk::CellRendererText::new();
    let column = gtk::TreeViewColumn::new();
    column.pack_start(&renderer, true);
    column.set_title("Name");
    column.add_attribute(&renderer, "text", Columns::Name as i32);
    column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
    column.set_fixed_width(250);
    column.set_sort_column_id(Columns::Name as i32);
    treeview.append_column(&column);

    // Column for size
    let renderer = gtk::CellRendererText::new();
    renderer.set_property("xalign", 1.0 as f32);
    let column = gtk::TreeViewColumn::new();
    column.pack_start(&renderer, true);
    column.set_title("Size");
    column.set_alignment(1.0);
    column.add_attribute(&renderer, "text", Columns::Size as i32);
    column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
    column.set_fixed_width(90);
    column.set_sort_column_id(Columns::Size as i32);
    treeview.append_column(&column);

    // Column for modified date
    let renderer = gtk::CellRendererText::new();
    let column = gtk::TreeViewColumn::new();
    column.pack_start(&renderer, true);
    column.set_title("Modified");
    column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
    column.set_fixed_width(140);
    column.set_sort_column_id(Columns::Modified as i32);
    column.set_cell_data_func(
        &renderer,
        Some(Box::new(|_col, ren, model, iter| {
            let modified = model.value(iter, Columns::Modified as i32);
            let modified = modified.get::<u64>().unwrap_or(0);
            let dt: DateTime<Local> = Local.timestamp_opt(modified as i64, 0).unwrap();
            let modified_text = dt.format("%d-%m-%Y %H:%M:%S").to_string();
            ren.set_property("text", modified_text);
        })),
    );
    treeview.append_column(&column);
}
