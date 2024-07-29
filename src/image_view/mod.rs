mod imp;

use eog::{Image, ZoomMode};
use glib::IsA;
use gtk::glib;

glib::wrapper! {
    pub struct ImageView(ObjectSubclass<imp::ImageViewImp>)
        @extends gtk::Bin, gtk::Container, gtk::Widget, @implements gtk::Buildable;
}

impl ImageView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn image(&self) -> Option<Image> {
        None
    }

    pub fn set_image(&self, _image: &impl IsA<Image>) {}

    pub fn set_image_post(&self) {}

    pub fn set_image_pre(&self, _image: &impl IsA<Image>) {}

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

// eog.set_scroll_wheel_zoom(true);
// eog.set_zoom_mode(eog::ZoomMode::Fill);
// eog.set_image_pre();
// eog.set_image_post();
// eog.image()
