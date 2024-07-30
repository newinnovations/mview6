pub mod draw;
pub mod io;
pub mod view;

use glib::IsA;
use view::ZoomMode;

pub struct Image {}

#[allow(unused_variables)]
impl Image {
    pub fn new_surface(
        surface: &cairo::Surface,
        src_x: i32,
        src_y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        Image {}
    }
    pub fn new_image_surface(surface: &cairo::ImageSurface) -> Self {
        Image {}
    }
    pub fn new_pixbuf(pixbuf: &gdk_pixbuf::Pixbuf) -> Self {
        Image {}
    }
    pub fn new_stream(stream: &impl IsA<gio::InputStream>) -> Result<Self, glib::Error> {
        Ok(Image {})
    }
    pub fn id(&self) -> i32 {
        0
    }
    pub fn pixbuf(&self) -> Option<gdk_pixbuf::Pixbuf> {
        None
    }
    pub fn modified(&self) {}
    pub fn rotate(&self, angle: i32) {}
    pub fn set_zoom_mode(&self, mode: ZoomMode) {}
}
