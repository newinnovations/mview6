use super::MViewWindowImp;

impl MViewWindowImp {
    pub(super) fn on_mouse_press(&self, position: (f64, f64)) {
        let w = self.widgets();
        if let Some(current) = w.file_view.current() {
            let (x, y) = position;
            let (x_offset, y_offset) = w.image_view.offset();
            let (x, y) = (x + x_offset, y + y_offset);
            let backend = self.backend.borrow();
            if let Some((new_backend, goto)) = backend.click(&current, x, y) {
                drop(backend);
                self.set_backend(new_backend, goto, false);
            }
        }
    }
}
