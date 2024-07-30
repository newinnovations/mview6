mod imp;

use gdk_pixbuf::Pixbuf;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib;

use super::Image;

glib::wrapper! {
    pub struct ImageView(ObjectSubclass<imp::ImageViewImp>)
        @extends gtk::Bin, gtk::Container, gtk::Widget, @implements gtk::Buildable;
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

impl ImageView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_image(&self, image: Image) {
        let mut p = self.imp().p.borrow_mut();
        p.image = image;
        p.create_surface();
        drop(p);
        self.imp().redraw();
    }

    pub fn image_modified(&self) {
        let mut p = self.imp().p.borrow_mut();
        p.create_surface();
        drop(p);
        self.imp().redraw();
    }

    pub fn set_image_post(&self) {}

    pub fn set_image_pre(&self, image: Image) {
        self.set_image(image);
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        ZoomMode::NoZoom
    }

    pub fn set_zoom_mode(&self, _mode: ZoomMode) {}

    pub fn set_scroll_wheel_zoom(&self, _scroll_wheel_zoom: bool) {}

    pub fn apply_zoom(&self, _zoom_mode: ZoomMode) {}

    pub fn offset(&self) -> (f64, f64) {
        let p = self.imp().p.borrow();
        (p.xofs, p.yofs)
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
    }
}
