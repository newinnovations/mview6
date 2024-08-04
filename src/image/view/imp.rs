use std::{
    cell::RefCell,
    sync::OnceLock,
    time::{Duration, SystemTime},
};

use crate::image::{draw::transparency_background, Image};
use gdk::EventMask;
use glib::{
    clone, ffi::g_source_remove, result_from_gboolean, subclass::Signal, BoolError, ObjectExt,
    Propagation, SourceId, StaticType,
};
use gtk::{
    glib::{self, ControlFlow},
    prelude::{WidgetExt, WidgetExtManual},
    subclass::prelude::*,
};

use super::{
    data::{ImageViewData, ZoomState, QUALITY_HIGH, QUALITY_LOW, ZOOM_MULTIPLIER},
    ImageView, ViewCursor,
};

pub const SIGNAL_VIEW_RESIZED: &str = "view-resized";

#[derive(Default)]
pub struct ImageViewImp {
    pub(super) data: RefCell<ImageViewData>,
    animation_timeout_id: RefCell<Option<SourceId>>,
    hq_redraw_timeout_id: RefCell<Option<SourceId>>,
    resize_notify_timeout_id: RefCell<Option<SourceId>>,
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
    pub fn cancel_animation(&self) {
        if let Some(id) = self.animation_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {}", e);
            }
        }
    }

    pub fn schedule_animation(&self, image: &Image, ts_previous_cb: SystemTime) {
        if image.is_animation() {
            if let Some(interval) = image.animation_delay_time(ts_previous_cb) {
                // dbg!(interval);
                let current = self
                    .animation_timeout_id
                    .replace(Some(glib::timeout_add_local(
                        interval,
                        clone!(@weak self as imp => @default-return ControlFlow::Break, move || {
                            imp.animation_cb();
                            ControlFlow::Break
                        }),
                    )));
                assert!(current.is_none())
            }
        }
    }

    fn animation_cb(&self) {
        let start = SystemTime::now();
        self.animation_timeout_id.replace(None);
        let mut p = self.data.borrow_mut();
        if p.image.animation_advance(SystemTime::now()) {
            let rotation = p.rotation;
            p.image.rotate(rotation);
            p.create_surface();
            self.schedule_animation(&p.image, start);
            p.redraw(QUALITY_LOW);
        }
    }

    fn cancel_hq_redraw(&self) {
        if let Some(id) = self.hq_redraw_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {}", e);
            }
        }
    }

    fn schedule_hq_redraw(&self) {
        self.hq_redraw_timeout_id
            .replace(Some(glib::timeout_add_local(
                Duration::from_millis(100),
                clone!(@weak self as imp => @default-return ControlFlow::Break, move || {
                    imp.hq_redraw_timeout_id.replace(None);
                    let mut p = imp.data.borrow_mut();
                    p.redraw(QUALITY_HIGH);
                    ControlFlow::Break
                }),
            )));
    }

    fn cancel_resize_notify(&self) {
        if let Some(id) = self.resize_notify_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {}", e);
            }
        }
    }

    fn schedule_resize_notify(&self) {
        self.resize_notify_timeout_id
            .replace(Some(glib::timeout_add_local(
                Duration::from_millis(300),
                clone!(@weak self as imp => @default-return ControlFlow::Break, move || {
                    imp.resize_notify_timeout_id.replace(None);
                    println!("Notify of resize");
                    let obj = imp.obj();
                    let allocation = obj.allocation();
                    obj.emit_by_name::<()>(SIGNAL_VIEW_RESIZED, &[&allocation.width(), &allocation.height()]);
                    // let mut p = imp.data.borrow_mut();
                    // p.redraw(QUALITY_HIGH);
                    ControlFlow::Break
                }),
            )));
    }
}

impl ObjectImpl for ImageViewImp {
    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![Signal::builder(SIGNAL_VIEW_RESIZED)
                .param_types([i32::static_type(), i32::static_type()])
                .build()]
        })
    }

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
        self.data.borrow_mut().view = Some(view.clone());
    }
}

impl WidgetImpl for ImageViewImp {
    fn realize(&self) {
        self.parent_realize();
        if let Some(window) = &self.obj().window() {
            let mut p = self.data.borrow_mut();
            p.transparency_background = transparency_background(window).ok();
        }
    }

    /// Display size changed
    fn configure_event(&self, _event: &gdk::EventConfigure) -> Propagation {
        self.cancel_resize_notify();
        let mut p = self.data.borrow_mut();
        p.apply_zoom();
        self.schedule_resize_notify();
        Propagation::Proceed
    }

    fn button_press_event(&self, event: &gdk::EventButton) -> Propagation {
        let mut p = self.data.borrow_mut();
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
        let mut p = self.data.borrow_mut();
        if p.drag.is_some() {
            p.drag = None;
            self.obj().set_cursor(ViewCursor::Normal);
            p.redraw(QUALITY_HIGH);
            Propagation::Stop
        } else {
            self.parent_button_release_event(event)
        }
    }

    fn motion_notify_event(&self, event: &gdk::EventMotion) -> Propagation {
        let mut p = self.data.borrow_mut();
        if let Some((drag_x, drag_y)) = p.drag {
            let (position_x, position_y) = event.position();
            p.xofs = drag_x - position_x;
            p.yofs = drag_y - position_y;
            p.redraw(QUALITY_LOW);
            Propagation::Stop
        } else {
            self.parent_motion_notify_event(event)
        }
    }

    fn scroll_event(&self, event: &gdk::EventScroll) -> Propagation {
        self.cancel_hq_redraw();
        let mut p = self.data.borrow_mut();
        if p.image.is_movable() {
            let zoom = match event.direction() {
                gdk::ScrollDirection::Up => p.zoom * ZOOM_MULTIPLIER,
                gdk::ScrollDirection::Down => p.zoom / ZOOM_MULTIPLIER,
                _ => p.zoom,
            };
            p.update_zoom(zoom, event.position());
            self.schedule_hq_redraw();
        }
        self.parent_scroll_event(event)
    }

    fn draw(&self, cr: &cairo::Context) -> Propagation {
        let start = SystemTime::now();
        let p = self.data.borrow();

        let (xofs, yofs, scaled_width, scaled_height) = p.image_coords();

        /* Paint the background */
        let allocation = self.obj().allocation();
        cr.rectangle(
            0.0,
            0.0,
            allocation.width() as f64,
            allocation.height() as f64,
        );

        cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
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

        cr.scale(p.zoom, p.zoom);
        if let Some(surface) = p.surface.as_ref() {
            let _ = cr.set_source_surface(surface, xofs / p.zoom, yofs / p.zoom);
        }
        cr.source().set_extend(cairo::Extend::Pad);
        if p.zoom_state() != ZoomState::NoZoom {
            cr.source().set_filter(p.quality);
        }
        let _ = cr.paint();

        if let Ok(d) = start.elapsed() {
            let elapsed = d.as_secs() as f64 * 1e3 + d.subsec_nanos() as f64 * 1e-6;
            println!("drawn in {:7.1} ms ({:?})", elapsed, p.quality);
        };

        Propagation::Stop
    }
}

impl DrawingAreaImpl for ImageViewImp {}
