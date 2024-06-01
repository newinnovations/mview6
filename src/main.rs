use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;
use gdk::glib::ObjectExt;
use gtk::glib;
use gtk::prelude::ApplicationExt;
use gtk::prelude::ApplicationExtManual;
use gtk::prelude::ContainerExt;
use gtk::prelude::GtkListStoreExtManual;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::ScrolledWindowExt;
use gtk::prelude::TreeModelExt;
use gtk::prelude::TreeSelectionExt;
use gtk::prelude::TreeViewColumnExt;
use gtk::prelude::TreeViewExt;
use gtk::prelude::WidgetExt;

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::rc::Rc;
use std::time::UNIX_EPOCH;

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
        let cat: u32 = 0;

        store.insert_with_values(
            None,
            &[
                (0, &cat),
                (1, &filename),
                (2, &file_size),
                (3, &modified),
            ],
        );
    }
    Ok(())
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("List Store");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(280, 250);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    window.add(&vbox);

    let label = gtk::Label::new(Some(
        "This is the bug list (note: not based on real data, it would be \
         nice to have a nice ODBC interface to bugzilla or so, though).",
    ));
    vbox.add(&label);

    let sw = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    sw.set_shadow_type(gtk::ShadowType::EtchedIn);
    sw.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    vbox.add(&sw);

    let model = Rc::new(create_model());
    let treeview = gtk::TreeView::with_model(&*model);
    treeview.set_vexpand(true);
    // treeview.set_search_column(Columns::Description as i32);

    sw.add(&treeview);

    add_columns(&treeview);

    treeview.connect_cursor_changed(|tv| {
        let selection = tv.selection();
        if let Some((model, iter)) = selection.selected() {
            println!(
                "Hello '{}' from row {}",
                model
                    .value(&iter, 1)
                    .get::<String>()
                    .expect("Treeview selection, column 1: mandatory value not found"),
                model
                    .value(&iter, 2)
                    .get::<u64>()
                    .expect("Treeview selection, column 0")
            );
        }
    });

    window.connect_key_press_event(move |_s, e| {
        println!("Key {}", e.keycode().unwrap());
        match e.keyval() {
            gdk::keys::constants::z |
            gdk::keys::constants::Left => {
                treeview.emit_move_cursor(gtk::MovementStep::DisplayLines, -1);
            }
            gdk::keys::constants::x |
            gdk::keys::constants::Right => {
                treeview.emit_move_cursor(gtk::MovementStep::DisplayLines, 1);
            }
            gdk::keys::constants::Page_Up => {
                treeview.emit_move_cursor(gtk::MovementStep::Pages, -1);
            }
            gdk::keys::constants::Page_Down => {
                treeview.emit_move_cursor(gtk::MovementStep::Pages, 1);
            }
            gdk::keys::constants::Home => {
                treeview.emit_move_cursor(gtk::MovementStep::BufferEnds, -1);
            }
            gdk::keys::constants::End => {
                treeview.emit_move_cursor(gtk::MovementStep::BufferEnds, 1);
            }
            gdk::keys::constants::Up => {
                let (tp, col) = treeview.cursor();
                if let Some(mut tp) = tp {
                    for _ in 0..5 {
                        tp.prev();
                    }
                    treeview.set_cursor(&tp, col.as_ref(), false);
                }
            }
            gdk::keys::constants::Down => {
                let (tp, col) = treeview.cursor();
                if let Some(mut tp) = tp {
                    for _ in 0..5 {
                        tp.next();
                    }
                    treeview.set_cursor(&tp, col.as_ref(), false);
                }
            }
            _ => (),
        }
        glib::Propagation::Stop
    });

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
    // column.add_attribute(&renderer, "text", Columns::ModifiedText as i32);
    column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
    column.set_fixed_width(140);
    column.set_sort_column_id(Columns::Modified as i32);
    column.set_cell_data_func(&renderer, Some(Box::new(
        |_col, ren, model, iter| {
            let modified = model.value(iter, Columns::Modified as i32);
            let modified = modified.get::<u64>().unwrap_or(0);
            let dt: DateTime<Local> = Local.timestamp_opt(modified as i64, 0).unwrap();
            let modified_text = dt.format("%d-%m-%Y %H:%M:%S").to_string();
            ren.set_property("text", modified_text);
        })));
    treeview.append_column(&column);

    // Column for file/direcory
    let renderer = gtk::CellRendererText::new();
    let column = gtk::TreeViewColumn::new();
    column.pack_start(&renderer, true);
    column.set_title("Name");
    column.add_attribute(&renderer, "text", Columns::Name as i32);
    column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
    column.set_fixed_width(50);
    column.set_sort_column_id(Columns::Name as i32);
    treeview.append_column(&column);
}
