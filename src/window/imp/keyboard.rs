use super::MViewWindowImp;

use eog::ScrollViewExt;
use gdk::EventKey;
use gtk::{prelude::*, subclass::prelude::*};

use crate::filelistview::FileListViewExt;

impl MViewWindowImp {
    pub(super) fn on_key_press(&self, e: &EventKey) {
        println!("Key {}", e.keycode().unwrap());
        let w = self.widgets.get().unwrap();
        w.treeview.set_has_focus(true);
        match e.keyval() {
            gdk::keys::constants::q => {
                self.obj().close();
            }
            gdk::keys::constants::space => {
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
            gdk::keys::constants::f => {
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
                if let Some(subdir) = &w.treeview.filename() {
                    let mut filelist = w.file_list.borrow_mut();
                    let newstore = filelist.enter(&subdir);
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
            gdk::keys::constants::m => {
                if w.sv.zoom_mode() == eog::ZoomMode::Max {
                    w.sv.set_zoom_mode(eog::ZoomMode::Fill);
                } else {
                    w.sv.set_zoom_mode(eog::ZoomMode::Max);
                }
            }
            gdk::keys::constants::z | gdk::keys::constants::Left => {
                w.treeview
                    .emit_move_cursor(gtk::MovementStep::DisplayLines, -1);
            }
            gdk::keys::constants::x | gdk::keys::constants::Right => {
                w.treeview
                    .emit_move_cursor(gtk::MovementStep::DisplayLines, 1);
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
            gdk::keys::constants::Up => {
                let (tp, col) = w.treeview.cursor();
                if let Some(mut tp) = tp {
                    println!("tp: {:?}", tp.indices());
                    // TreePath::from_indicesv(&[3]);
                    // let n = tp.indices().get(0).unwrap().to_owned();
                    // let m = w.treeview.model().unwrap();
                    // let i = m.iter_nth_child(None, n).unwrap();
                    // println!(
                    //     "Current = {}",
                    //     model
                    //         .value(&i, Columns::Name as i32)
                    //         .get::<String>()
                    //         .unwrap_or("??".to_string())
                    // );
                    for _ in 0..1 {
                        tp.prev();
                    }
                    w.treeview.set_cursor(&tp, col.as_ref(), false);
                }
            }
            gdk::keys::constants::Down => {
                let (tp, col) = w.treeview.cursor();
                if let Some(mut tp) = tp {
                    println!("tp: {:?}", tp.indices());
                    for _ in 0..1 {
                        tp.next();
                    }
                    w.treeview.set_cursor(&tp, col.as_ref(), false);
                }
            }
            _ => (),
        }
    }
}
