use super::MViewWindowImp;

use eog::ScrollViewExt;
use gdk::EventKey;
use gtk::{prelude::*, subclass::prelude::*};

use crate::filelistview::{Direction, FileListViewExt, Filter};

impl MViewWindowImp {
    pub(super) fn on_key_press(&self, e: &EventKey) {
        println!("Key {}", e.keycode().unwrap());
        let w = self.widgets.get().unwrap();
        w.treeview.set_has_focus(true);
        match e.keyval() {
            gdk::keys::constants::q => {
                self.obj().close();
            }
            gdk::keys::constants::space | gdk::keys::constants::KP_Divide => {
                if w.file_window.is_visible() {
                    w.file_window.set_visible(false);
                    w.hbox.set_spacing(0);
                    self.obj().set_border_width(0);
                } else {
                    w.file_window.set_visible(true);
                    w.hbox.set_spacing(8);
                    self.obj().set_border_width(10);
                }
            }
            gdk::keys::constants::f | gdk::keys::constants::KP_Multiply => {
                if self.fs.get() {
                    self.obj().unfullscreen();
                    self.fs.set(false);
                } else {
                    w.file_window.set_visible(false);
                    w.hbox.set_spacing(0);
                    self.obj().set_border_width(0);
                    self.obj().fullscreen();
                    self.fs.set(true);
                }
            }
            gdk::keys::constants::Return => {
                if let Some(subdir) = &w.treeview.current_filename() {
                    let mut filelist = w.file_list.borrow_mut();
                    let newstore = filelist.enter(subdir);
                    drop(filelist);
                    if newstore.is_some() {
                        w.treeview.set_model(newstore.as_ref());
                        w.treeview.goto_first();
                    }
                }
            }
            gdk::keys::constants::BackSpace => {
                let mut filelist = w.file_list.borrow_mut();
                let newstore = filelist.leave();
                drop(filelist);
                w.treeview.set_model(newstore.as_ref());
                w.treeview.goto_first();
            }
            gdk::keys::constants::d => {
                w.treeview.write();
            }
            gdk::keys::constants::o => {
                if w.sv.zoom_mode() == eog::ZoomMode::Fit {
                    w.sv.set_zoom_mode(eog::ZoomMode::None);
                } else {
                    w.sv.set_zoom_mode(eog::ZoomMode::Fit);
                }
            }
            gdk::keys::constants::m | gdk::keys::constants::KP_0 => {
                if w.sv.zoom_mode() == eog::ZoomMode::Max {
                    w.sv.set_zoom_mode(eog::ZoomMode::Fill);
                } else {
                    w.sv.set_zoom_mode(eog::ZoomMode::Max);
                }
            }
            gdk::keys::constants::z | gdk::keys::constants::Left | gdk::keys::constants::KP_4 => {
                w.treeview.navigate(Direction::Up, Filter::Image, 1);
            }
            gdk::keys::constants::x | gdk::keys::constants::Right | gdk::keys::constants::KP_6 => {
                w.treeview.navigate(Direction::Down, Filter::Image, 1);
            }
            gdk::keys::constants::a => {
                w.treeview.navigate(Direction::Up, Filter::Favorite, 1);
            }
            gdk::keys::constants::s => {
                w.treeview.navigate(Direction::Down, Filter::Favorite, 1);
            }
            gdk::keys::constants::Z => {
                w.treeview.navigate(Direction::Up, Filter::None, 1);
            }
            gdk::keys::constants::X => {
                w.treeview.navigate(Direction::Down, Filter::None, 1);
            }
            gdk::keys::constants::Up | gdk::keys::constants::KP_8 => {
                w.treeview.navigate(Direction::Up, Filter::Image, 5);
            }
            gdk::keys::constants::Down | gdk::keys::constants::KP_2 => {
                w.treeview.navigate(Direction::Down, Filter::Image, 5);
            }
            gdk::keys::constants::Page_Up => {
                w.treeview.emit_move_cursor(gtk::MovementStep::Pages, -1);
            }
            gdk::keys::constants::Page_Down => {
                w.treeview.emit_move_cursor(gtk::MovementStep::Pages, 1);
            }
            gdk::keys::constants::Home => {
                w.treeview
                    .emit_move_cursor(gtk::MovementStep::BufferEnds, -1);
            }
            gdk::keys::constants::End => {
                w.treeview
                    .emit_move_cursor(gtk::MovementStep::BufferEnds, 1);
            }
            _ => (),
        }
    }
}
