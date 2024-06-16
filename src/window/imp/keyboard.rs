use super::MViewWindowImp;

use eog::ScrollViewExt;
use gdk::EventKey;
use gtk::{prelude::*, subclass::prelude::*};

use crate::filelistview::{Direction, FileListViewExt, Filter};

impl MViewWindowImp {
    pub(super) fn on_key_press(&self, e: &EventKey) {
        let w = self.widgets.get().unwrap();
        w.file_list_view.set_has_focus(true);
        match e.keyval() {
            gdk::keys::constants::q => {
                self.obj().close();
            }
            gdk::keys::constants::space | gdk::keys::constants::KP_Divide => {
                if w.files_widget.is_visible() {
                    w.files_widget.set_visible(false);
                    w.hbox.set_spacing(0);
                    self.obj().set_border_width(0);
                } else {
                    w.files_widget.set_visible(true);
                    w.hbox.set_spacing(8);
                    self.obj().set_border_width(10);
                }
            }
            gdk::keys::constants::f | gdk::keys::constants::KP_Multiply => {
                if self.full_screen.get() {
                    self.obj().unfullscreen();
                    self.full_screen.set(false);
                } else {
                    w.files_widget.set_visible(false);
                    w.hbox.set_spacing(0);
                    self.obj().set_border_width(0);
                    self.obj().fullscreen();
                    self.full_screen.set(true);
                }
            }
            gdk::keys::constants::Return => {
                self.dir_enter();
            }
            gdk::keys::constants::BackSpace => {
                self.dir_leave();
            }
            gdk::keys::constants::n => {
                if w.eog.zoom_mode() == eog::ZoomMode::Fit {
                    w.eog.set_zoom_mode(eog::ZoomMode::None);
                } else {
                    w.eog.set_zoom_mode(eog::ZoomMode::Fit);
                }
            }
            gdk::keys::constants::m | gdk::keys::constants::KP_0 => {
                if w.eog.zoom_mode() == eog::ZoomMode::Max {
                    w.eog.set_zoom_mode(eog::ZoomMode::Fill);
                } else {
                    w.eog.set_zoom_mode(eog::ZoomMode::Max);
                }
            }
            gdk::keys::constants::minus | gdk::keys::constants::KP_Subtract => {
                w.file_list_view.set_unsorted();
                if w.file_list_view
                    .favorite(&w.file_list.borrow().directory(), Direction::Down)
                {
                    w.file_list_view.navigate(Direction::Down, Filter::Image, 1);
                }
            }
            gdk::keys::constants::equal | gdk::keys::constants::KP_Add => {
                w.file_list_view.set_unsorted();
                if w.file_list_view
                    .favorite(&w.file_list.borrow().directory(), Direction::Up)
                {
                    w.file_list_view.navigate(Direction::Down, Filter::Image, 1);
                }
            }
            gdk::keys::constants::z | gdk::keys::constants::Left | gdk::keys::constants::KP_4 => {
                let filter = if w.files_widget.is_visible() {
                    Filter::None
                } else {
                    Filter::Image
                };
                w.file_list_view.navigate(Direction::Up, filter, 1);
            }
            gdk::keys::constants::x | gdk::keys::constants::Right | gdk::keys::constants::KP_6 => {
                let filter = if w.files_widget.is_visible() {
                    Filter::None
                } else {
                    Filter::Image
                };
                w.file_list_view.navigate(Direction::Down, filter, 1);
            }
            gdk::keys::constants::a => {
                w.file_list_view
                    .navigate(Direction::Up, Filter::Favorite, 1);
            }
            gdk::keys::constants::s => {
                w.file_list_view
                    .navigate(Direction::Down, Filter::Favorite, 1);
            }
            gdk::keys::constants::Z => {
                w.file_list_view.navigate(Direction::Up, Filter::None, 1);
            }
            gdk::keys::constants::X => {
                w.file_list_view.navigate(Direction::Down, Filter::None, 1);
            }
            gdk::keys::constants::Up | gdk::keys::constants::KP_8 => {
                let filter = if w.files_widget.is_visible() {
                    Filter::None
                } else {
                    Filter::Image
                };
                w.file_list_view.navigate(Direction::Up, filter, 5);
            }
            gdk::keys::constants::Down | gdk::keys::constants::KP_2 => {
                let filter = if w.files_widget.is_visible() {
                    Filter::None
                } else {
                    Filter::Image
                };
                w.file_list_view.navigate(Direction::Down, filter, 5);
            }
            gdk::keys::constants::Page_Up => {
                w.file_list_view
                    .emit_move_cursor(gtk::MovementStep::Pages, -1);
            }
            gdk::keys::constants::Page_Down => {
                w.file_list_view
                    .emit_move_cursor(gtk::MovementStep::Pages, 1);
            }
            gdk::keys::constants::Home => {
                w.file_list_view
                    .emit_move_cursor(gtk::MovementStep::BufferEnds, -1);
            }
            gdk::keys::constants::End => {
                w.file_list_view
                    .emit_move_cursor(gtk::MovementStep::BufferEnds, 1);
            }
            _ => (),
        }
    }
}
