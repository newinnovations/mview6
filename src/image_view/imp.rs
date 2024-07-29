use glib::Propagation;
use gtk::{glib, subclass::prelude::*};

use super::ImageView;

#[derive(Debug, Default)]
pub struct ImageViewImp {}

#[glib::object_subclass]
impl ObjectSubclass for ImageViewImp {
    const NAME: &'static str = "ImageWindow";
    type Type = ImageView;
    type ParentType = gtk::Bin;
}

impl ImageViewImp {}

impl ObjectImpl for ImageViewImp {
    fn constructed(&self) {
        self.parent_constructed();
        println!("constructed");
    }

}
impl WidgetImpl for ImageViewImp {
    fn draw(&self, cr: &cairo::Context) -> Propagation {
        println!("draw");
        self.parent_draw(cr)
    }
}
impl ContainerImpl for ImageViewImp {}
impl BinImpl for ImageViewImp {}
