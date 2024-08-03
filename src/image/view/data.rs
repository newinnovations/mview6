use cairo::{Filter, Surface};
use gdk::prelude::GdkPixbufExt;
use gtk::prelude::WidgetExt;

use crate::image::Image;

use super::{ImageView, ZoomMode};

pub const MAX_ZOOM_FACTOR: f64 = 30.0;
pub const MIN_ZOOM_FACTOR: f64 = 0.02;
pub const ZOOM_MULTIPLIER: f64 = 1.05;
pub const QUALITY_HIGH: Filter = Filter::Bilinear;
pub const QUALITY_LOW: Filter = Filter::Nearest;

#[derive(Debug)]
pub struct ImageViewData {
    pub image: Image,
    pub zoom_mode: ZoomMode,
    pub xofs: f64,
    pub yofs: f64,
    pub rotation: i32,
    pub surface: Option<Surface>,
    pub transparency_background: Option<Surface>,
    pub view: Option<ImageView>,
    pub zoom: f64,
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

impl ImageViewData {
    //     if (w > MAX_IMAGE_SIZE || h > MAX_IMAGE_SIZE) {
    //         g_warning ("Image dimensions too large to process");
    //         w = 50;
    //         h = 50;
    //         surface = gdk_window_create_similar_image_surface (
    //                 gtk_widget_get_window (view->priv->display),
    //                 CAIRO_FORMAT_ARGB32, w, h, 1.0);
    pub(super) fn create_surface(&mut self) {
        if let (Some(pixbuf), Some(view)) = (&self.image.pixbuf, &self.view) {
            self.surface = pixbuf.create_surface(1, view.window().as_ref());
        } else {
            self.surface = None;
        }
    }

    fn compute_scaled_size(&self, zoom: f64) -> (f64, f64) {
        if let Some(pixbuf) = &self.image.pixbuf {
            (
                (pixbuf.width() as f64 * zoom).round(), // Remove round() ??
                (pixbuf.height() as f64 * zoom).round(),
            )
        } else {
            (0.0, 0.0)
        }
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

    pub fn apply_zoom(&mut self, update_zoom: bool) {
        if let (Some(pixbuf), Some(view)) = (&self.image.pixbuf, &self.view) {
            let allocation = view.allocation();
            let allocation_width = allocation.width() as f64;
            let allocation_height = allocation.height() as f64;
            let src_width = pixbuf.width() as f64;
            let src_height = pixbuf.height() as f64;

            if update_zoom {
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
            }

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
