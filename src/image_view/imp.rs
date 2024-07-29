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

impl ObjectImpl for ImageViewImp {}
impl WidgetImpl for ImageViewImp {}
impl ContainerImpl for ImageViewImp {}
impl BinImpl for ImageViewImp {}
