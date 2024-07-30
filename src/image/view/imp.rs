use std::cell::RefCell;

use cairo::Surface;
use gdk::prelude::GdkPixbufExt;
use glib::Propagation;
use gtk::{
    glib,
    prelude::{ContainerExt, WidgetExt},
    subclass::prelude::*,
    DrawingArea,
};

use crate::image::Image;

use super::ImageView;

#[derive(Debug, Default)]
pub(super) struct ImageViewPrivate {
    pub(super) image: Image,
    // pub(super) pixbuf: Option<Pixbuf>,
    surface: Option<Surface>,
    drawing_area: Option<DrawingArea>,
    zoom: f64,
    pub(super) xofs: f64,
    pub(super) yofs: f64,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
enum ZoomState {
    NoZoom,
    ZoomedIn,
    ZoomedOut,
}
// impl Default for ImageViewPrivate {
//     fn default() -> Self {
//         Self {
//             pixbuf: Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, 10, 10).unwrap(),
//         }
//     }
// }

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
        if let (Some(pixbuf), Some(drawing_area)) = (&self.image.pixbuf, &self.drawing_area) {
            self.surface = pixbuf.create_surface(1, drawing_area.window().as_ref());
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

    fn eog_scroll_view_get_image_coords(&self) -> (f64, f64, f64, f64) {
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
}

#[derive(Debug, Default)]
pub struct ImageViewImp {
    pub(super) p: RefCell<ImageViewPrivate>,
}

#[glib::object_subclass]
impl ObjectSubclass for ImageViewImp {
    const NAME: &'static str = "ImageWindow";
    type Type = ImageView;
    type ParentType = gtk::Bin;
}

impl ImageViewImp {
    pub fn redraw(&self) {
        println!("redraw");
        let p = self.p.borrow();
        p.drawing_area.as_ref().unwrap().queue_draw();
    }
}

impl ObjectImpl for ImageViewImp {
    fn constructed(&self) {
        self.parent_constructed();
        let mut p = self.p.borrow_mut();
        p.zoom = 1.0;

        let drawing_area = DrawingArea::builder().can_focus(true).build();

        // gtk_drag_source_set (priv->display, GDK_BUTTON1_MASK,
        //     target_table, G_N_ELEMENTS (target_table),
        //     GDK_ACTION_COPY | GDK_ACTION_MOVE |
        //     GDK_ACTION_LINK | GDK_ACTION_ASK);

        self.obj().add(&drawing_area);
        drawing_area.set_expand(true);
        p.drawing_area = Some(drawing_area);

        println!("constructed");
    }
}

impl WidgetImpl for ImageViewImp {
    fn draw(&self, cr: &cairo::Context) -> Propagation {
        // let mut x = self.p.borrow_mut();
        // x.pixbuf = Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, 10, 10);
        println!("draw");
        let p = self.p.borrow();

        // if let Some(pixbuf) = &p.pixbuf {
        let (xofs, yofs, scaled_width, scaled_height) = p.eog_scroll_view_get_image_coords();
        dbg!(p.zoom, xofs, yofs, scaled_width, scaled_height);

        /* Paint the background */
        let allocation = self.obj().allocation();
        dbg!(allocation);
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

        // if (gdk_pixbuf_get_has_alpha (priv->pixbuf)) {
        //     if (priv->background_surface == NULL) {
        //         priv->background_surface = create_background_surface (view);
        //     }
        //     cairo_set_source_surface (cr, priv->background_surface, xofs, yofs);
        //     cairo_pattern_set_extend (cairo_get_source (cr), CAIRO_EXTEND_REPEAT);
        //     cairo_rectangle (cr, xofs, yofs, scaled_width, scaled_height);
        //     cairo_fill (cr);
        // }

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
            cr.source().set_filter(cairo::Filter::Good);
        }

        let _ = cr.paint();

        Propagation::Proceed
        // } else {
        //     self.parent_draw(cr)
        // }
    }
}

impl ContainerImpl for ImageViewImp {}
impl BinImpl for ImageViewImp {}
