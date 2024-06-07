use crate::filelist::Columns;
use crate::filelist_view;
use chrono::{DateTime, Local, TimeZone};
use glib::ObjectExt;
use gtk::glib;
use gtk::prelude::{CellRendererExt, TreeModelExt, TreeViewColumnExt, TreeViewExt};
use gtk::subclass::prelude::*;

#[derive(Debug, Default)]
pub struct FileListView {
    // widgets: OnceCell<WindowWidgets>,
    // counter: Cell<u64>,
}

#[glib::object_subclass]
impl ObjectSubclass for FileListView {
    const NAME: &'static str = "FileListView";
    type Type = filelist_view::FileListView;
    type ParentType = gtk::TreeView;
}

impl WidgetImpl for FileListView {}

impl ContainerImpl for FileListView {}

impl TreeViewImpl for FileListView {}

impl ObjectImpl for FileListView {
    // Here we are overriding the glib::Object::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
    fn constructed(&self) {
        self.parent_constructed();
        let instance = self.obj();

        // Column for category
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Cat");
        column.add_attribute(&renderer, "text", Columns::Cat as i32);
        column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        column.set_fixed_width(40);
        column.set_sort_column_id(Columns::Cat as i32);
        instance.append_column(&column);

        // Column for file/direcory
        let renderer_txt = gtk::CellRendererText::new();
        let renderer_icon = gtk::CellRendererPixbuf::new();
        renderer_icon.set_padding(6, 0);
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer_icon, false);
        column.pack_start(&renderer_txt, true);
        column.set_title("Name");
        column.add_attribute(&renderer_icon, "icon-name", Columns::Icon as i32);
        column.add_attribute(&renderer_txt, "text", Columns::Name as i32);
        column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        column.set_fixed_width(250);
        column.set_sort_column_id(Columns::Name as i32);
        instance.append_column(&column);

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
        instance.append_column(&column);

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
        instance.append_column(&column);
    }
}

impl FileListView {
    // fn on_increment_clicked(&self) {
    //     self.counter.set(self.counter.get() + 1);
    //     let w = self.widgets.get().unwrap();
    //     w.label
    //         .set_text(&format!("Counter is {}", self.counter.get()));
    // }
}
