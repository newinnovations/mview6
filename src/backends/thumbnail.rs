use std::cell::RefCell;

use cairo::{Context, Format, ImageSurface};
use eog::{Image, ImageExt};
use gdk::prelude::GdkContextExt;
use gdk_pixbuf::Pixbuf;
use gtk::{
    prelude::{GtkListStoreExtManual, TreeModelExt},
    ListStore, TreeIter,
};

use crate::{category::Category, draw::draw, error::MviewResult};

use super::{empty_store, Backend, Columns, TreeModelMviewExt};

pub struct Thumbnail {
    size: i32,
    sheet_x: i32,
    sheet_y: i32,
    separator_x: i32,
    separator_y: i32,
    parent: RefCell<Box<dyn Backend>>,
}

impl Thumbnail {
    pub fn new() -> Self {
        Thumbnail {
            size: 175,
            sheet_x: 1920,
            sheet_y: 1080,
            separator_x: 4,
            separator_y: 4,
            parent: RefCell::new(<dyn Backend>::invalid()),
        }
    }

    pub fn capacity_x(&self) -> i32 {
        (self.sheet_x + self.separator_x) / (self.size + self.separator_x)
    }
    pub fn capacity_y(&self) -> i32 {
        (self.sheet_y + self.separator_y) / (self.size + self.separator_y)
    }

    pub fn capacity(&self) -> i32 {
        self.capacity_x() * self.capacity_y()
    }

    pub fn sheet(&self) -> MviewResult<Image> {
        let backend = self.parent.borrow();
        let store = backend.store();
        // let caption = "sheet 1 of 100";
        let offset_x = (self.sheet_x - self.capacity_x() * (self.size + self.separator_x)
            + self.separator_x)
            / 2;
        let offset_y = (self.sheet_y - self.capacity_y() * (self.size + self.separator_y)
            + self.separator_y)
            / 2;

        let surface = ImageSurface::create(Format::ARgb32, self.sheet_x, self.sheet_y)?;
        let context = Context::new(&surface)?;

        context.set_source_rgb(1.0, 0.2, 0.4);
        // context.set_source_rgb(0.0, 0.0, 0.0);
        context.paint()?;

        // // graphics.setBackground(Color.white);
        // // graphics.setFont(new Font("LucidaSans", Font.BOLD, 10));

        // context.select_font_face(
        //     "LucidaSans",
        //     cairo::FontSlant::Normal,
        //     cairo::FontWeight::Bold,
        // ); //Bold);
        // let extends = context.text_extents(&caption)?;
        // dbg!(&extends);

        // // FontMetrics fm = graphics.getFontMetrics();
        // // int txtW = fm.stringWidth(caption);
        // // int txtH = fm.getHeight();
        // let txtW = extends.width();
        // let txtH = extends.height();
        // // graphics.clearRect(sheetX - txtW - 15, 5, txtW + 10, txtH + 7);
        // // graphics.setPaint(Color.black);
        // // graphics.drawString(caption, sheetX - txtW - 10, txtH + 5);

        let mut done = false;
        if let Some(iter) = store.iter_nth_child(None, 0) {
            for y in 0..self.capacity_y() {
                if done {
                    break;
                }
                for x in 0..self.capacity_x() {
                    let f = store.filename(&iter);
                    dbg!(&f);

                    if let Ok(thumb) = backend.thumb(&store, &iter) {
                        let thumb = pixbuf_scale(thumb, self.size);
                        let tox = (self.size - thumb.width()) / 2;
                        let toy = (self.size - thumb.height()) / 2;
                        let topleft_x = offset_x + tox + x * (self.size + self.separator_x);
                        let topleft_y = offset_y + toy + y * (self.size + self.separator_y);
                        context.set_source_pixbuf(&thumb, topleft_x as f64, topleft_y as f64);
                        context.paint()?;
                    }

                    if !store.iter_next(&iter) {
                        done = true;
                        break;
                    }
                }
            }
        }

        let image = Image::new_image_surface(&surface);
        image.set_zoom_mode(eog::ZoomMode::None);

        Ok(image)
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

        let pages = (1 + num_items/self.capacity()) as u32;

        let store = empty_store();


        // let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        // let modified = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
        // let file_size = metadata.len();
        let cat = Category::Direcory;

        for page in 0..pages {
            let name = format!("Thumbnail page {}", page + 1);
            store.insert_with_values(
                None,
                &[
                    (Columns::Cat as u32, &cat.id()),
                    (Columns::Icon as u32, &cat.icon()),
                    (Columns::Name as u32, &name),
                    (Columns::Index as u32, &page),
                    // (Columns::Folder as u32, &entry.folder),
                    // (Columns::Size as u32, &file_size),
                    // (Columns::Modified as u32, &modified),
                ],
            );

        }
        store
    }

    fn enter(&self, _model: ListStore, _iter: TreeIter) -> Box<dyn Backend> {
        Box::new(Thumbnail::new())
    }

    fn leave(&self) -> (Box<dyn Backend>, String) {
        (Box::new(Thumbnail::new()), "/".to_string())
    }

    fn image(&self, _model: ListStore, _iter: TreeIter) -> Image {
        // let store = self.parent.borrow().store();

        // let num_items = store.iter_n_children(None);
        // dbg!(num_items);

        // if let Some(iter) = store.iter_nth_child(None, 0) {
        //     if let Ok(image) = sheet(store, iter) {
        //         return image;
        //     }
        // }

        // let num_items = store.iter_n_children(None);
        // dbg!(num_items);

        // if let Some(iter) = store.iter_nth_child(None, 0) {
        if let Ok(image) = self.sheet() {
            return image;
        }
        // }

        draw("Thumbnail sheet failed").unwrap()
    }

    fn set_parent(&self, parent: Box<dyn Backend>) {
        self.parent.replace(parent);
    }
}

pub fn pixbuf_scale(pixbuf: Pixbuf, size: i32) -> Pixbuf {
    let width = pixbuf.width();
    let height = pixbuf.height();

    let (thumb_width, thumb_height) = if width > height {
        (size, height * size / width)
    } else {
        (width * size / height, size)
    };

    pixbuf
        .scale_simple(thumb_width, thumb_height, gdk_pixbuf::InterpType::Bilinear)
        .unwrap()
}
