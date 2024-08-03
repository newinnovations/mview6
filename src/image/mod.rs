pub mod animation;
pub mod draw;
pub mod provider;
pub mod view;

use animation::{Animation, AnimationFrames};
use cairo::ImageSurface;
use gdk::{
    ffi::gdk_pixbuf_get_from_surface,
    prelude::{PixbufAnimationExt, PixbufAnimationExtManual, PixbufLoaderExt},
};
use gdk_pixbuf::{Pixbuf, PixbufLoader, PixbufRotation};
use gio::{prelude::InputStreamExt, Cancellable};
use glib::{translate::from_glib_full, IsA};
use view::ZoomMode;

use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::SystemTime,
};

static IMAGE_ID: AtomicU32 = AtomicU32::new(1);

fn get_image_id() -> u32 {
    IMAGE_ID.fetch_add(1, Ordering::SeqCst);
    IMAGE_ID.load(Ordering::SeqCst)
}

#[derive(Default, Debug)]
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

    pub fn new_pixbuf(pixbuf: Pixbuf, zoom_mode: ZoomMode) -> Self {
        Image {
            id: get_image_id(),
            pixbuf: Some(pixbuf),
            animation: Animation::None,
            zoom_mode,
        }
    }

    pub fn new_animation_frames(animation_frames: AnimationFrames, zoom_mode: ZoomMode) -> Self {
        Image {
            id: get_image_id(),
            pixbuf: animation_frames.pixbuf_get(0),
            animation: Animation::WebP(Box::new(animation_frames)),
            zoom_mode,
        }
    }

    pub fn new_stream(
        stream: &impl IsA<gio::InputStream>,
        zoom_mode: ZoomMode,
    ) -> Result<Self, glib::Error> {
        let cancellable = Option::<Cancellable>::None.as_ref();
        let loader = PixbufLoader::new();
        loop {
            let b = stream.read_bytes(65536, cancellable)?;
            if b.len() == 0 {
                break;
            }
            loader.write_bytes(&b)?;
        }
        loader.close()?;
        stream.close(cancellable)?;
        let (pixbuf, animation) = if let Some(animation) = loader.animation() {
            if animation.is_static_image() {
                (animation.static_image(), Animation::None)
            } else {
                let iter = animation.iter(Some(SystemTime::now()));
                (Some(iter.pixbuf()), Animation::Gdk(iter))
            }
        } else {
            (None, Animation::None)
        };
        Ok(Image {
            id: get_image_id(),
            pixbuf,
            animation,
            zoom_mode,
        })
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