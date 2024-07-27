mod model;
pub mod processing;

use std::cell::RefCell;

use super::{Backend, Selection};
use crate::{
    category::Category,
    draw::thumbnail_sheet,
    filelistview::{Columns, Cursor},
    window::MViewWidgets,
};
use eog::{Image, ImageExt};
use gdk_pixbuf::Pixbuf;
use gtk::{
    prelude::{GtkListStoreExtManual, TreeModelExt},
    ListStore,
};
pub use model::{Message, TCommand, TEntry, TMessage, TReference, TResult, TResultOption, TTask};

#[derive(Debug)]
pub struct Thumbnail {
    size: i32,
    width: i32,
    height: i32,
    // calculated
    separator_x: i32,
    separator_y: i32,
    capacity_x: i32,
    capacity_y: i32,
    offset_x: i32,
    offset_y: i32,
    // references
    parent: RefCell<Box<dyn Backend>>,
    parent_pos: i32,
}

impl Default for Thumbnail {
    fn default() -> Self {
        Self::new(800, 600, 0, 175)
    }
}

impl Thumbnail {
    pub fn new(width: i32, height: i32, position: i32, size: i32) -> Self {
        let footer = 50;
        let min_separator = 5;

        let capacity_x = (width + min_separator) / (size + min_separator);
        let capacity_y = (height - footer + min_separator) / (size + min_separator);

        let separator_x = (width - capacity_x * size) / capacity_x;
        let separator_y = (height - footer - capacity_y * size) / capacity_y;

        let offset_x = (width - capacity_x * (size + separator_x) + separator_x) / 2;
        let offset_y = (height - footer - capacity_y * (size + separator_y) + separator_y) / 2;

        Thumbnail {
            size,
            width,
            height,
            separator_x,
            separator_y,
            capacity_x,
            capacity_y,
            offset_x,
            offset_y,
            parent: RefCell::new(<dyn Backend>::none()),
            parent_pos: position,
        }
    }

    pub fn capacity(&self) -> i32 {
        self.capacity_x * self.capacity_y
    }

    pub fn startpage(&self) -> Selection {
        Selection::Index(self.parent_pos as u32 / self.capacity() as u32)
    }

    pub fn sheet(&self, page: i32) -> Vec<TTask> {
        let backend = self.parent.borrow();
        let store = backend.store();

        let mut res = Vec::<TTask>::new();

        let mut done = false;
        let mut tid = 0;
        let start = page * self.capacity();
        if let Some(iter) = store.iter_nth_child(None, start) {
            let cursor = Cursor::new(store, iter, start);
            for row in 0..self.capacity_y {
                if done {
                    break;
                }
                for col in 0..self.capacity_x {
                    let source = backend.entry(&cursor);
                    if !matches!(source.reference, TReference::None) {
                        let task = TTask::new(
                            tid,
                            self.size as u32,
                            self.offset_x + col * (self.size + self.separator_x),
                            self.offset_y + row * (self.size + self.separator_y),
                            source,
                        );
                        res.push(task);
                        tid += 1;
                    }
                    if !cursor.next() {
                        done = true;
                        break;
                    }
                }
            }
        }

        res
    }
}

impl Backend for Thumbnail {
    fn class_name(&self) -> &str {
        "Thumbnail"
    }

    fn path(&self) -> &str {
        "/thumbnail"
    }

    fn store(&self) -> ListStore {
        let parent_store = self.parent.borrow().store();
        let num_items = parent_store.iter_n_children(None);
        let pages = (1 + num_items / self.capacity()) as u32;
        let store = Columns::store();
        let cat = Category::Image;

        for page in 0..pages {
            let name = format!("Thumbnail page {:7}", page + 1);
            store.insert_with_values(
                None,
                &[
                    (Columns::Cat as u32, &cat.id()),
                    (Columns::Icon as u32, &cat.icon()),
                    (Columns::Name as u32, &name),
                    (Columns::Index as u32, &page),
                ],
            );
        }
        store
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        let parent_backend = self.parent.replace(<dyn Backend>::none());
        (parent_backend, Selection::None)
    }

    fn image(&self, w: &MViewWidgets, cursor: &Cursor) -> Image {
        let page = cursor.index();
        let caption = format!("sheet {} of {}", page + 1, cursor.store_size());

        let image = match thumbnail_sheet(self.width, self.height, self.offset_x, &caption) {
            Ok(image) => image,
            Err(_) => {
                let pixbuf = Pixbuf::new(
                    gdk_pixbuf::Colorspace::Rgb,
                    true,
                    8,
                    self.width,
                    self.height,
                )
                .unwrap();
                pixbuf.fill(0x202020ff);
                let image = Image::new_pixbuf(&pixbuf);
                image.set_zoom_mode(eog::ZoomMode::None);
                image
            }
        };
        let id = image.id();
        let command = TCommand::new(id, self.sheet(page as i32));

        let _ = w.sender.send(Message::Command(command));

        image
    }

    fn set_parent(&self, parent: Box<dyn Backend>) {
        self.parent.replace(parent);
    }

    fn is_thumbnail(&self) -> bool {
        true
    }

    fn click(&self, current: &Cursor, x: f64, y: f64) -> Option<(Box<dyn Backend>, Selection)> {
        let x = (x as i32 - self.offset_x) / (self.size + self.separator_x);
        let y = (y as i32 - self.offset_y) / (self.size + self.separator_y);

        if x < 0 || y < 0 || x >= self.capacity_x || y >= self.capacity_y {
            return None;
        }

        let page = current.index() as i32;
        let pos = page * self.capacity() + y * self.capacity_x + x;

        let backend = self.parent.borrow();
        let store = backend.store();
        if let Some(iter) = store.iter_nth_child(None, pos) {
            let cursor = Cursor::new(store, iter, pos);
            let source = backend.entry(&cursor).reference;
            drop(backend);
            match source {
                TReference::FileReference(src) => Some((
                    self.parent.replace(<dyn Backend>::none()),
                    Selection::Name(src.filename()),
                )),
                TReference::ZipReference(src) => Some((
                    self.parent.replace(<dyn Backend>::none()),
                    Selection::Index(src.index()),
                )),
                TReference::RarReference(src) => Some((
                    self.parent.replace(<dyn Backend>::none()),
                    Selection::Name(src.selection()),
                )),
                TReference::None => None,
            }
        } else {
            None
        }
    }
}
