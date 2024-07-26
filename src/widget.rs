use gdk::{prelude::MonitorExt, Display, Rectangle};
use glib::IsA;
use gtk::prelude::WidgetExt;

pub trait MViewWidgetExt: IsA<gtk::Widget> {
    fn display_size(&self) -> gdk::Rectangle;
}

impl<O: IsA<gtk::Widget>> MViewWidgetExt for O {
    fn display_size(&self) -> gdk::Rectangle {
        if let Some(display) = Display::default() {
            if let Some(window) = self.window() {
                if let Some(monitor) = display.monitor_at_window(&window) {
                    return monitor.workarea();
                }
            }
        }
        Rectangle::new(0, 0, 800, 600)
    }
}
