mod data;
mod imp;

use std::time::SystemTime;

use data::QUALITY_HIGH;
use gdk::{Cursor, CursorType};
use gdk_pixbuf::Pixbuf;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::{glib, prelude::WidgetExt};

use super::Image;

glib::wrapper! {
    pub struct ImageView(ObjectSubclass<imp::ImageViewImp>)
        @extends gtk::DrawingArea, gtk::Widget, @implements gtk::Buildable;
}

impl Default for ImageView {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum ZoomMode {
    #[default]
    NotSpecified,
    NoZoom,
    Fit,
    Fill,
    Max,
}

pub enum ViewCursor {
    Normal,
    Hidden,
    Drag,
}

impl ImageView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_image(&self, image: Image) {
        self.set_image_pre(image);
        self.set_image_post();
    }

    pub fn set_image_pre(&self, image: Image) {
        let mut p = self.imp().data.borrow_mut();
        self.imp().cancel_animation();
        p.image = image;
        p.rotation = 0;
    }

    pub fn set_image_post(&self) {
        let mut p = self.imp().data.borrow_mut();
        p.create_surface();
        self.imp().schedule_animation(&p.image, SystemTime::now());
        p.apply_zoom();
    }

    pub fn image_modified(&self) {
        let mut p = self.imp().data.borrow_mut();
        p.create_surface();
        p.redraw(QUALITY_HIGH);
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        let p = self.imp().data.borrow();
        p.zoom_mode
    }

    pub fn set_zoom_mode(&self, mode: ZoomMode) {
        let mut p = self.imp().data.borrow_mut();
        p.zoom_mode = mode;
        p.apply_zoom();
    }

    pub fn offset(&self) -> (f64, f64) {
        let p = self.imp().data.borrow();
        (p.xofs, p.yofs)
    }

    pub fn set_cursor(&self, view_cursor: ViewCursor) {
        if let Some(toplevel) = self.toplevel() {
            if let Some(window) = toplevel.window() {
                let display = toplevel.display();
                let cursor = match view_cursor {
                    ViewCursor::Normal => None,
                    ViewCursor::Hidden => Cursor::for_display(&display, CursorType::BlankCursor),
                    ViewCursor::Drag => Cursor::for_display(&display, CursorType::Fleur),
                };
                window.set_cursor(cursor.as_ref());
            }
        }
    }

    // Operations on image

    pub fn image_id(&self) -> u32 {
        self.imp().data.borrow().image.id()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        let p = self.imp().data.borrow();
        p.image.draw_pixbuf(pixbuf, dest_x, dest_y);
    }

    pub fn rotate(&self, angle: i32) {
        let mut p = self.imp().data.borrow_mut();
        let center = p.center();
        p.rotation = (p.rotation + angle).rem_euclid(360);
        p.image.rotate(angle);
        p.create_surface();
        p.move_center_to(center);
        p.redraw(QUALITY_HIGH);
    }
}
