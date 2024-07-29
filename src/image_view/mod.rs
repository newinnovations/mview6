pub mod image;
mod imp;

use gtk::glib;
use image::Image;

glib::wrapper! {
    pub struct ImageView(ObjectSubclass<imp::ImageViewImp>)
        @extends gtk::Bin, gtk::Container, gtk::Widget, @implements gtk::Buildable;
}

impl Default for ImageView {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum ZoomMode {
    Unspecified,
    None,
    Fit,
    Fill,
    Max,
}

impl ImageView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn image(&self) -> Option<Image> {
        None
    }

    pub fn set_image(&self, _image: Image) {}

    pub fn set_image_post(&self) {}

    pub fn set_image_pre(&self, _image: Image) {}

    pub fn zoom_mode(&self) -> ZoomMode {
        ZoomMode::None
    }

    pub fn set_zoom_mode(&self, _mode: ZoomMode) {}

    pub fn set_scroll_wheel_zoom(&self, _scroll_wheel_zoom: bool) {}

    pub fn apply_zoom(&self, _zoom_mode: ZoomMode) {}

    pub fn x_offset(&self) -> i32 {
        0
    }
    pub fn y_offset(&self) -> i32 {
        0
    }
}
