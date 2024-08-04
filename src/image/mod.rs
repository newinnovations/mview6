pub mod animation;
pub mod colors;
pub mod draw;
pub mod provider;
pub mod view;

use animation::Animation;
use cairo::ImageSurface;
use gdk::ffi::gdk_pixbuf_get_from_surface;
use gdk_pixbuf::{Pixbuf, PixbufRotation};
use glib::translate::from_glib_full;
use std::sync::atomic::{AtomicU32, Ordering};
use view::ZoomMode;

static IMAGE_ID: AtomicU32 = AtomicU32::new(1);

fn get_image_id() -> u32 {
    IMAGE_ID.fetch_add(1, Ordering::SeqCst);
    IMAGE_ID.load(Ordering::SeqCst)
}

#[derive(Default)]
pub struct Image {
    id: u32,
    pixbuf: Option<Pixbuf>,
    animation: Animation,
    zoom_mode: ZoomMode,
}

impl Image {
    pub fn new_surface(surface: &ImageSurface, zoom_mode: ZoomMode) -> Self {
        let pixbuf: Option<Pixbuf> = unsafe {
            from_glib_full(gdk_pixbuf_get_from_surface(
                surface.as_ref().to_raw_none(),
                0,
                0,
                surface.width(),
                surface.height(),
            ))
        };
        Image {
            id: get_image_id(),
            pixbuf,
            animation: Animation::None,
            zoom_mode,
        }
    }

    pub fn new_pixbuf(pixbuf: Option<Pixbuf>) -> Self {
        Image {
            id: get_image_id(),
            pixbuf,
            animation: Animation::None,
            zoom_mode: ZoomMode::NotSpecified,
        }
    }

    pub fn new_animation(animation: Animation) -> Self {
        let pixbuf = match &animation {
            Animation::None => None,
            Animation::Gdk(a) => Some(a.pixbuf()),
            Animation::WebPFile(a) => a.pixbuf_get(0),
            Animation::WebPMemory(a) => a.pixbuf_get(0),
        };
        Image {
            id: get_image_id(),
            pixbuf,
            animation,
            zoom_mode: ZoomMode::NotSpecified,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn rotate(&mut self, angle: i32) {
        let rotation = match angle {
            90 => PixbufRotation::Counterclockwise,
            180 => PixbufRotation::Upsidedown,
            270 => PixbufRotation::Clockwise,
            _ => {
                return;
            }
        };
        if let Some(pixbuf) = &self.pixbuf {
            self.pixbuf = pixbuf.rotate_simple(rotation);
        }
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        self.zoom_mode
    }

    pub fn is_movable(&self) -> bool {
        self.zoom_mode != ZoomMode::NoZoom
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        if let Some(my_pixpuf) = &self.pixbuf {
            pixbuf.copy_area(
                0,
                0,
                pixbuf.width(),
                pixbuf.height(),
                my_pixpuf,
                dest_x,
                dest_y,
            );
        }
    }
}
