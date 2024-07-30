use gdk::{EventButton, EventMotion};

use crate::filelistview::FileListViewExt;

use super::MViewWindowImp;

impl MViewWindowImp {
    pub(super) fn on_mouse_move(&self, _e: &EventMotion) {
        // dbg!(e);
    }

    pub(super) fn on_mouse_press(&self, e: &EventButton) {
        let w = self.widgets.get().unwrap();
        if let Some(current) = w.file_list_view.current() {
            let (x, y) = e.position();
            let (x_offset, y_offset) = w.eog.offset();
            let (x, y) = (x + x_offset, y + y_offset);

            let backend = w.backend.borrow();

            if let Some((new_backend, goto)) = backend.click(&current, x, y) {
                drop(backend);
                self.set_backend(new_backend, goto, false);
            }
        }
    }
}
