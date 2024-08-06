use std::slice;

use cairo::{Filter, ImageSurface};
use gdk_pixbuf::{
    ffi::{gdk_pixbuf_get_byte_length, gdk_pixbuf_read_pixels},
    Pixbuf,
};
use glib::translate::ToGlibPtr;
use gtk4::prelude::WidgetExt;

use crate::{image::Image, performance::Performance};

use super::{ImageView, ZoomMode};

pub const MAX_ZOOM_FACTOR: f64 = 30.0;
pub const MIN_ZOOM_FACTOR: f64 = 0.02;
pub const ZOOM_MULTIPLIER: f64 = 1.05;
pub const QUALITY_HIGH: Filter = Filter::Bilinear;
pub const QUALITY_LOW: Filter = Filter::Nearest;

pub struct ImageViewData {
    pub image: Image,
    pub zoom_mode: ZoomMode,
    pub xofs: f64,
    pub yofs: f64,
    pub rotation: i32,
    pub surface: Option<ImageSurface>,
    pub transparency_background: Option<ImageSurface>,
    pub view: Option<ImageView>,
    pub zoom: f64,
    pub mouse_position: (f64, f64),
    pub drag: Option<(f64, f64)>,
    pub quality: cairo::Filter,
}

impl Default for ImageViewData {
    fn default() -> Self {
        Self {
            image: Image::default(),
            zoom_mode: ZoomMode::NotSpecified,
            xofs: 0.0,
            yofs: 0.0,
            rotation: 0,
            surface: None,
            transparency_background: None,
            view: None,
            zoom: 1.0,
            mouse_position: (0.0, 0.0),
            drag: None,
            quality: QUALITY_HIGH,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum ZoomState {
    NoZoom,
    ZoomedIn,
    ZoomedOut,
}

// https://users.rust-lang.org/t/converting-a-bgra-u8-to-rgb-u8-n-for-images/67938
fn create_surface(p: &Pixbuf) -> Option<ImageSurface> {
    let duration = Performance::start();

    let width = p.width() as usize;
    let height = p.height() as usize;
    let pixbuf_stride = p.rowstride() as usize;

    let surface_stride = 4 * width;
    let mut surface_data = vec![0_u8; height * surface_stride];

    unsafe {
        // gain access without the copy of pixbuf memory
        let pixbuf_data_raw = gdk_pixbuf_read_pixels(p.to_glib_none().0);
        let pixbuf_data_len = gdk_pixbuf_get_byte_length(p.to_glib_none().0);
        let pixbuf_data = slice::from_raw_parts(pixbuf_data_raw, pixbuf_data_len);

        if p.has_alpha() {
            for (src_row, dst_row) in pixbuf_data
                .chunks_exact(pixbuf_stride)
                .zip(surface_data.chunks_exact_mut(surface_stride))
            {
                for (src, dst) in src_row.chunks_exact(4).zip(dst_row.chunks_exact_mut(4)) {
                    dst[0] = src[2];
                    dst[1] = src[1];
                    dst[2] = src[0];
                    dst[3] = src[3];
                }
            }
        } else {
            for (src_row, dst_row) in pixbuf_data
                .chunks_exact(pixbuf_stride)
                .zip(surface_data.chunks_exact_mut(surface_stride))
            {
                for (src, dst) in src_row.chunks_exact(3).zip(dst_row.chunks_exact_mut(4)) {
                    dst[0] = src[2];
                    dst[1] = src[1];
                    dst[2] = src[0];
                    dst[3] = 255;
                }
            }
        }
    }

    let surface = match ImageSurface::create_for_data(
        surface_data,
        cairo::Format::ARgb32,
        width as i32,
        height as i32,
        surface_stride as i32,
    ) {
        Ok(s) => Some(s),
        Err(e) => {
            dbg!(e);
            None
        }
    };

    duration.elapsed("surface");

    surface
}

impl ImageViewData {
    pub(super) fn create_surface(&mut self) {
        if let Some(pixbuf) = &self.image.pixbuf {
            // self.surface = pixbuf.create_surface(1, view.window().as_ref());
            self.surface = create_surface(pixbuf);
        } else {
            self.surface = None;
        }
    }

    fn compute_scaled_size(&self, zoom: f64) -> (f64, f64) {
        if let Some(pixbuf) = &self.image.pixbuf {
            (pixbuf.width() as f64 * zoom, pixbuf.height() as f64 * zoom)
        } else {
            (0.0, 0.0)
        }
    }

    pub fn center(&self) -> (f64, f64) {
        let (w, h) = self.compute_scaled_size(self.zoom);
        (w / 2.0 - self.xofs, h / 2.0 - self.yofs)
    }

    pub fn move_center_to(&mut self, center: (f64, f64)) {
        let (cx, cy) = center;
        let (w, h) = self.compute_scaled_size(self.zoom);
        self.xofs = w / 2.0 - cx;
        self.yofs = h / 2.0 - cy;
    }

    pub fn image_coords(&self) -> (f64, f64, f64, f64) {
        let (scaled_width, scaled_height) = self.compute_scaled_size(self.zoom);
        (-self.xofs, -self.yofs, scaled_width, scaled_height)
    }

    pub fn zoom_state(&self) -> ZoomState {
        if self.zoom > 1.0 + 1.0e-6 {
            ZoomState::ZoomedIn
        } else if self.zoom < 1.0 - 1.0e-6 {
            ZoomState::ZoomedOut
        } else {
            ZoomState::NoZoom
        }
    }

    pub fn redraw(&mut self, quality: Filter) {
        if let Some(view) = &self.view {
            self.quality = quality;
            view.queue_draw();
        }
    }

    pub fn apply_zoom(&mut self) {
        if let (Some(pixbuf), Some(view)) = (&self.image.pixbuf, &self.view) {
            let allocation = view.allocation();
            let allocation_width = allocation.width() as f64;
            let allocation_height = allocation.height() as f64;
            let src_width = pixbuf.width() as f64;
            let src_height = pixbuf.height() as f64;

            let zoom_mode = if self.image.zoom_mode == ZoomMode::NotSpecified {
                if self.zoom_mode == ZoomMode::NotSpecified {
                    ZoomMode::NoZoom
                } else {
                    self.zoom_mode
                }
            } else {
                self.image.zoom_mode
            };

            let zoom = if zoom_mode == ZoomMode::NoZoom {
                1.0
            } else {
                let zoom1 = allocation_width / src_width;
                let zoom2 = allocation_height / src_height;
                if zoom_mode == ZoomMode::Max {
                    if zoom1 > zoom2 {
                        zoom1
                    } else {
                        zoom2
                    }
                } else if zoom_mode == ZoomMode::Fit
                    && allocation_width > src_width
                    && allocation_height > src_height
                {
                    1.0
                } else if zoom1 > zoom2 {
                    zoom2
                } else {
                    zoom1
                }
            };
            self.zoom = zoom.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);

            self.xofs = ((self.zoom * src_width - allocation_width) / 2.0).round();
            self.yofs = ((self.zoom * src_height - allocation_height) / 2.0).round();
        }
        self.redraw(QUALITY_HIGH);
    }

    pub fn update_zoom(&mut self, zoom: f64, anchor: (f64, f64)) {
        let old_zoom = self.zoom;
        let new_zoom = zoom.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
        if new_zoom == old_zoom {
            return;
        }
        let (anchor_x, anchor_y) = anchor;
        let view_cx = (self.xofs + anchor_x) / old_zoom;
        let view_cy = (self.yofs + anchor_y) / old_zoom;
        self.xofs = view_cx * new_zoom - anchor_x;
        self.yofs = view_cy * new_zoom - anchor_y;
        self.zoom = new_zoom;
        if self.drag.is_some() {
            self.drag = Some((anchor_x + self.xofs, anchor_y + self.yofs))
        }
        self.redraw(QUALITY_LOW);
    }
}
