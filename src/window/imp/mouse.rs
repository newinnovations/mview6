use eog::ScrollViewExt;
use gdk::{EventButton, EventMotion};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub(super) fn on_mouse_move(&self, _e: &EventMotion) {
        // dbg!(e);
    }

    pub(super) fn on_mouse_press(&self, e: &EventButton) {
        let w = self.widgets.get().unwrap();

        let (x, y) = e.position();
        let (x, y) = (x + w.eog.x_offset() as f64, y + w.eog.y_offset() as f64);

        let backend = w.backend.borrow();

        if let Some((new_backend, goto)) = backend.click(x, y) {
            drop(backend);
            self.set_backend(new_backend, goto.as_deref());
        }
    }
}
