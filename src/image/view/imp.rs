use std::{cell::RefCell, time::SystemTime};

use cairo::Surface;
use gdk::{prelude::GdkPixbufExt, EventMask};
use glib::{clone, ffi::g_source_remove, result_from_gboolean, BoolError, Propagation, SourceId};
use gtk::{
    glib,
    prelude::{WidgetExt, WidgetExtManual},
    subclass::prelude::*,
};

use crate::image::{draw::transparency_background, Image};

use super::{ImageView, ViewCursor, ZoomMode};

const MAX_ZOOM_FACTOR: f64 = 20.0;
const MIN_ZOOM_FACTOR: f64 = 0.02;
const ZOOM_MULTIPLIER: f64 = 1.05;

#[derive(Debug, Default)]
pub(super) struct ImageViewPrivate {
    pub(super) image: Image,
    pub(super) zoom_mode: ZoomMode,
    pub(super) xofs: f64,
    pub(super) yofs: f64,
    pub(super) rotation: i32,
    surface: Option<Surface>,
    transparency_background: Option<Surface>,
    view: Option<ImageView>,
    zoom: f64,
    drag: Option<(f64, f64)>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
enum ZoomState {
    NoZoom,
    ZoomedIn,
    ZoomedOut,
}

impl ImageViewPrivate {
    // create_surface_from_pixbuf (EogScrollView *view, GdkPixbuf *pixbuf)
    // {
    //     cairo_surface_t *surface;
    //     gint w, h;

    //     w = gdk_pixbuf_get_width (pixbuf);
    //     h = gdk_pixbuf_get_height (pixbuf);

    //     if (w > MAX_IMAGE_SIZE || h > MAX_IMAGE_SIZE) {
    //         g_warning ("Image dimensions too large to process");
    //         w = 50;
    //         h = 50;

    //         surface = gdk_window_create_similar_image_surface (
    //                 gtk_widget_get_window (view->priv->display),
    //                 CAIRO_FORMAT_ARGB32, w, h, 1.0);
    //     } else {
    //         surface = gdk_cairo_surface_create_from_pixbuf (pixbuf, 1.0,
    //                 gtk_widget_get_window (view->priv->display));
    //     }

    //     return surface;
    // }

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

    fn image_coords(&self) -> (f64, f64, f64, f64) {
        let (scaled_width, scaled_height) = self.compute_scaled_size(self.zoom);
        (-self.xofs, -self.yofs, scaled_width, scaled_height)
    }

    fn zoom_state(&self) -> ZoomState {
        if self.zoom > 1.0 + 1.0e-6 {
            ZoomState::ZoomedIn
        } else if self.zoom < 1.0 - 1.0e-6 {
            ZoomState::ZoomedOut
        } else {
            ZoomState::NoZoom
        }
    }

    pub fn redraw(&self) {
        if let Some(view) = &self.view {
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

            view.queue_draw();
        }
    }

    fn update_zoom(&mut self, zoom: f64, anchor: (f64, f64)) {
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
        self.redraw();
    }
}

#[derive(Debug, Default)]
pub struct ImageViewImp {
    pub(super) p: RefCell<ImageViewPrivate>,
    animation_timeout_id: RefCell<Option<SourceId>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ImageViewImp {
    const NAME: &'static str = "ImageWindow";
    type Type = ImageView;
    type ParentType = gtk::DrawingArea;
}

fn remove_source_id(id: SourceId) -> Result<(), BoolError> {
    unsafe { result_from_gboolean!(g_source_remove(id.as_raw()), "Failed to remove source") }
}

impl ImageViewImp {
    pub fn animation(&self, image: &Image) {
        if let Some(id) = self.animation_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {}", e);
            }
        }
        if let Some(animation) = &image.animation {
            if let Some(interval) = animation.delay_time() {
                dbg!(interval);
                self.animation_timeout_id
                    .replace(Some(glib::timeout_add_local(
                        interval,
                        clone!(@weak self as imp => @default-panic, move || {
                            imp.animation_cb();
                            glib::ControlFlow::Break
                        }),
                    )));
            }
        }
    }

    fn animation_cb(&self) {
        let mut p = self.p.borrow_mut();
        if let Some(animation) = &p.image.animation {
            if animation.advance(SystemTime::now()) {
                let rotation = p.rotation;
                p.image.pixbuf = Some(animation.pixbuf());
                p.image.rotate(rotation);
                p.create_surface();
                self.animation(&p.image);
                p.redraw();
            }
        }
    }
}

impl ObjectImpl for ImageViewImp {
    fn constructed(&self) {
        self.parent_constructed();
        let view = self.obj();
        view.set_can_focus(true);
        view.set_expand(true);
        view.add_events(
            EventMask::BUTTON_PRESS_MASK
                | EventMask::BUTTON_RELEASE_MASK
                | EventMask::POINTER_MOTION_MASK
                | EventMask::SCROLL_MASK,
        );
        let mut p = self.p.borrow_mut();
        p.zoom = 1.0;
        p.view = Some(view.clone());
    }
}

impl WidgetImpl for ImageViewImp {
    fn realize(&self) {
        println!("realize");
        self.parent_realize();
        if let Some(window) = &self.obj().window() {
            let mut p = self.p.borrow_mut();
            p.transparency_background = transparency_background(window).ok();
        } else {
            println!("realize without window")
        }
    }

    /// Display size changed
    fn configure_event(&self, _event: &gdk::EventConfigure) -> Propagation {
        let mut p = self.p.borrow_mut();
        p.apply_zoom(true);
        Propagation::Proceed
    }

    fn button_press_event(&self, event: &gdk::EventButton) -> Propagation {
        let mut p = self.p.borrow_mut();
        if p.drag.is_none() && event.button() == 1 && p.image.is_movable() {
            let (position_x, position_y) = event.position();
            p.drag = Some((position_x + p.xofs, position_y + p.yofs));
            self.obj().set_cursor(ViewCursor::Drag);
            Propagation::Stop
        } else {
            self.parent_button_press_event(event)
        }
    }

    fn button_release_event(&self, event: &gdk::EventButton) -> Propagation {
        let mut p = self.p.borrow_mut();
        if p.drag.is_some() {
            p.drag = None;
            self.obj().set_cursor(ViewCursor::Normal);
            Propagation::Stop
        } else {
            self.parent_button_release_event(event)
        }
    }

    fn motion_notify_event(&self, event: &gdk::EventMotion) -> Propagation {
        let mut p = self.p.borrow_mut();
        if let Some((drag_x, drag_y)) = p.drag {
            let (position_x, position_y) = event.position();
            p.xofs = drag_x - position_x;
            p.yofs = drag_y - position_y;
            p.redraw();
            Propagation::Stop
        } else {
            self.parent_motion_notify_event(event)
        }
    }

    fn scroll_event(&self, event: &gdk::EventScroll) -> Propagation {
        let mut p = self.p.borrow_mut();
        if p.image.is_movable() {
            let zoom = match event.direction() {
                gdk::ScrollDirection::Up => p.zoom * ZOOM_MULTIPLIER,
                gdk::ScrollDirection::Down => p.zoom / ZOOM_MULTIPLIER,
                _ => p.zoom,
            };
            p.update_zoom(zoom, event.position());
        }

        self.parent_scroll_event(event)
    }

    fn draw(&self, cr: &cairo::Context) -> Propagation {
        // let mut x = self.p.borrow_mut();
        // x.pixbuf = Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, 10, 10);
        println!("draw");
        let p = self.p.borrow();

        // if let Some(pixbuf) = &p.pixbuf {
        let (xofs, yofs, scaled_width, scaled_height) = p.image_coords();
        // dbg!(p.zoom, xofs, yofs, scaled_width, scaled_height);

        /* Paint the background */
        let allocation = self.obj().allocation();
        // dbg!(allocation);
        cr.rectangle(
            0.0,
            0.0,
            allocation.width() as f64,
            allocation.height() as f64,
        );
        cr.rectangle(xofs, yofs, scaled_width, scaled_height);
        cr.set_source_rgba(0.4, 0.2, 0.2, 1.0);
        cr.set_fill_rule(cairo::FillRule::EvenOdd);
        let _ = cr.fill();

        if let (Some(pixbuf), Some(transparency_background)) =
            (&p.image.pixbuf, &p.transparency_background)
        {
            if pixbuf.has_alpha() {
                let _ = cr.set_source_surface(transparency_background, xofs, yofs);
                cr.source().set_extend(cairo::Extend::Repeat);
                cr.rectangle(xofs, yofs, scaled_width, scaled_height);
                let _ = cr.fill();
            }
        }

        /* Make sure the image is only drawn as large as needed.
         * This is especially necessary for SVGs where there might
         * be more image data available outside the image boundaries.
         */
        cr.rectangle(xofs, yofs, scaled_width, scaled_height);
        cr.clip();

        // cairo_filter_t interp_type;

        // if(!DOUBLE_EQUAL(priv->zoom, 1.0) && priv->force_unfiltered)
        // {
        // 	interp_type = CAIRO_FILTER_NEAREST;
        // 	_set_hq_redraw_timeout(view);
        // }
        // else
        // {
        // 	if (is_zoomed_in (view))
        // 		interp_type = priv->interp_type_in;
        // 	else
        // 		interp_type = priv->interp_type_out;

        // 	_clear_hq_redraw_timeout (view);
        // 	priv->force_unfiltered = TRUE;
        // }

        // cairo_scale (cr, priv->zoom, priv->zoom);
        cr.scale(p.zoom, p.zoom);

        // cairo_set_source_surface (cr, priv->surface, xofs/priv->zoom, yofs/priv->zoom);
        if let Some(surface) = p.surface.as_ref() {
            let _ = cr.set_source_surface(surface, xofs / p.zoom, yofs / p.zoom);
        }

        // cairo_pattern_set_extend (cairo_get_source (cr), CAIRO_EXTEND_PAD);
        cr.source().set_extend(cairo::Extend::Pad);

        // if (is_zoomed_in (view) || is_zoomed_out (view))
        // 	cairo_pattern_set_filter (cairo_get_source (cr), interp_type);
        if p.zoom_state() != ZoomState::NoZoom {
            cr.source().set_filter(cairo::Filter::Fast);
        }

        let _ = cr.paint();

        Propagation::Stop
        // } else {
        //     self.parent_draw(cr)
        // }
    }
}

impl DrawingAreaImpl for ImageViewImp {}
