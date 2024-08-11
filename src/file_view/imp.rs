pub mod cursor;
pub mod model;
pub mod sort;

use std::cell::OnceCell;

use crate::file_view;
use chrono::{
    offset::LocalResult,
    {Local, TimeZone},
};
use glib::{
    object::ObjectExt,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::{ObjectSubclass, ObjectSubclassExt},
    },
};
use gtk4::{
    glib,
    prelude::TreeViewExt,
    subclass::{prelude::TreeViewImpl, widget::WidgetImpl},
    CellRendererPixbuf, CellRendererText, TreeView, TreeViewColumn, TreeViewColumnSizing,
};
use human_bytes::human_bytes;

use cursor::TreeModelMviewExt;
use model::Columns;

#[derive(Debug)]
#[allow(dead_code)]
struct FileViewColumns {
    category: TreeViewColumn,
    name: TreeViewColumn,
    size: TreeViewColumn,
    date: TreeViewColumn,
}

#[derive(Default)]
pub struct FileViewImp {
    columns: OnceCell<FileViewColumns>,
}

#[glib::object_subclass]
impl ObjectSubclass for FileViewImp {
    const NAME: &'static str = "FileListView";
    type Type = file_view::FileView;
    type ParentType = TreeView;
}

impl FileViewImp {
    pub(super) fn set_extended(&self, extended: bool) {
        let columns = self.columns.get().unwrap();
        if extended != columns.size.is_visible() {
            columns.size.set_visible(extended);
            columns.date.set_visible(extended);
        }
    }
}

impl ObjectImpl for FileViewImp {
    fn constructed(&self) {
        self.parent_constructed();
        let instance = self.obj();

        // Column for category
        let renderer = CellRendererPixbuf::new();
        let col_category = TreeViewColumn::new();
        col_category.pack_start(&renderer, true);
        // column.set_title("Cat");
        col_category.add_attribute(&renderer, "icon-name", Columns::Icon as i32);
        col_category.set_sizing(TreeViewColumnSizing::Fixed);
        col_category.set_fixed_width(30);
        col_category.set_sort_column_id(Columns::Cat as i32);
        instance.append_column(&col_category);

        // Column for file/direcory
        let renderer_txt = CellRendererText::new();
        // let renderer_icon = CellRendererPixbuf::new();
        // renderer_icon.set_padding(6, 0);
        let col_name = TreeViewColumn::new();
        // column.pack_start(&renderer_icon, false);
        col_name.pack_start(&renderer_txt, true);
        col_name.set_title("Name");
        // column.add_attribute(&renderer_icon, "icon-name", Columns::Icon as i32);
        col_name.add_attribute(&renderer_txt, "text", Columns::Name as i32);
        col_name.set_sizing(TreeViewColumnSizing::Fixed);
        col_name.set_fixed_width(300);
        col_name.set_sort_column_id(Columns::Name as i32);
        instance.append_column(&col_name);

        // Column for size
        let renderer = CellRendererText::new();
        renderer.set_property("xalign", 1.0_f32);
        let col_size = TreeViewColumn::new();
        col_size.pack_start(&renderer, true);
        col_size.set_title("Size");
        col_size.set_alignment(1.0);
        col_size.add_attribute(&renderer, "text", Columns::Size as i32);
        col_size.set_sizing(TreeViewColumnSizing::Fixed);
        col_size.set_fixed_width(90);
        col_size.set_sort_column_id(Columns::Size as i32);
        col_size.set_cell_data_func(&renderer, |_col, renderer, model, iter| {
            let size = model.size(iter);
            let modified_text = if size > 0 {
                human_bytes(size as f64)
            } else {
                String::default()
            };
            renderer.set_property("text", modified_text);
        });
        instance.append_column(&col_size);

        // Column for modified date
        let renderer = CellRendererText::new();
        let col_date = TreeViewColumn::new();
        col_date.pack_start(&renderer, true);
        col_date.set_title("Modified");
        col_date.set_sizing(TreeViewColumnSizing::Fixed);
        col_date.set_fixed_width(140);
        col_date.set_sort_column_id(Columns::Modified as i32);
        col_date.set_cell_data_func(&renderer, |_col, renderer, model, iter| {
            let modified = model.modified(iter);
            let modified_text = if modified > 0 {
                if let LocalResult::Single(dt) = Local.timestamp_opt(modified as i64, 0) {
                    dt.format("%d-%m-%Y %H:%M:%S").to_string()
                } else {
                    String::default()
                }
            } else {
                String::default()
            };
            renderer.set_property("text", modified_text);
        });
        instance.append_column(&col_date);

        self.columns
            .set(FileViewColumns {
                category: col_category,
                name: col_name,
                size: col_size,
                date: col_date,
            })
            .expect("Failed to store file list columns");
    }
}

impl WidgetImpl for FileViewImp {}

impl TreeViewImpl for FileViewImp {}

impl FileViewImp {}
