mod imp;

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
        let mut p = self.imp().p.borrow_mut();
        p.image = image;
        p.create_surface();
        self.imp().animation(&p.image); // start/stop animation if needed
        p.apply_zoom();
    }

    pub fn image_modified(&self) {
        let mut p = self.imp().p.borrow_mut();
        p.create_surface();
        p.redraw();
    }

    pub fn set_image_post(&self) {}

    pub fn set_image_pre(&self, image: Image) {
        self.set_image(image);
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        let p = self.imp().p.borrow();
        p.zoom_mode
    }

    pub fn set_zoom_mode(&self, mode: ZoomMode) {
        let mut p = self.imp().p.borrow_mut();
        p.zoom_mode = mode;
        p.apply_zoom();
    }

    pub fn offset(&self) -> (f64, f64) {
        let p = self.imp().p.borrow();
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
        self.imp().p.borrow().image.id()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        let p = self.imp().p.borrow();
        p.image.draw_pixbuf(pixbuf, dest_x, dest_y);
    }

    pub fn rotate(&self, angle: i32) {
        self.imp().p.borrow().image.rotate(angle);
        // w.eog.apply_zoom(w.eog.zoom_mode());
    }
}
