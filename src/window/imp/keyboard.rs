use super::{MViewWidgetExt, MViewWindowImp};

use eog::{ImageExt, ScrollViewExt};
use gdk::EventKey;
use gtk::{prelude::*, subclass::prelude::*, SortColumn};

use crate::{
    backends::{thumbnail::Thumbnail, Backend},
    filelistview::{Direction, FileListViewExt, Filter, Selection, Sort},
};

impl MViewWindowImp {
    pub(super) fn on_key_press(&self, e: &EventKey) {
        let w = self.widgets.get().unwrap();
        w.file_list_view.set_has_focus(true);
        match e.keyval() {
            gdk::keys::constants::q => {
                self.obj().close();
            }
            gdk::keys::constants::d => {
                if !w.backend.borrow().is_bookmarks() {
                    self.show_files_widget(true);
                    self.set_backend(<dyn Backend>::bookmarks(), Selection::None, true);
                }
            }
            gdk::keys::constants::t => {
                if w.backend.borrow().is_container() {
                    let position = if let Some(cursor) = w.file_list_view.current() {
                        cursor.position()
                    } else {
                        0
                    };
                    let display_size = self.obj().display_size();
                    let thumbnail = Thumbnail::new(
                        display_size.width(),
                        display_size.height(),
                        position,
                        self.thumbnail_size.get(),
                    );
                    let startpage = thumbnail.startpage();
                    let new_backend = <dyn Backend>::thumbnail(thumbnail);
                    new_backend.set_sort(&Sort::sort_on_category());
                    self.set_backend(new_backend, startpage, true);
                    self.show_files_widget(false);
                    self.obj().fullscreen();
                    self.full_screen.set(true);
                }
            }
            gdk::keys::constants::w
            | gdk::keys::constants::KP_7
            | gdk::keys::constants::KP_Home => {
                self.hop(Direction::Up);
            }
            gdk::keys::constants::e
            | gdk::keys::constants::KP_9
            | gdk::keys::constants::KP_Page_Up => {
                self.hop(Direction::Down);
            }
            gdk::keys::constants::space | gdk::keys::constants::KP_Divide => {
                self.show_files_widget(!w.files_widget.is_visible());
            }
            gdk::keys::constants::f | gdk::keys::constants::KP_Multiply => {
                if self.full_screen.get() {
                    self.obj().unfullscreen();
                    self.full_screen.set(false);
                } else {
                    self.show_files_widget(false);
                    self.obj().fullscreen();
                    self.full_screen.set(true);
                }
            }
            gdk::keys::constants::Escape => {
                self.obj().unfullscreen();
                self.full_screen.set(false);
            }
            gdk::keys::constants::r => {
                if let Some(image) = w.eog.image() {
                    image.rotate(270);
                    w.eog.apply_zoom(w.eog.zoom_mode());
                }
            }
            gdk::keys::constants::R => {
                if let Some(image) = w.eog.image() {
                    image.rotate(90);
                    w.eog.apply_zoom(w.eog.zoom_mode());
                }
            }
            gdk::keys::constants::Return => {
                self.dir_enter(None);
            }
            gdk::keys::constants::BackSpace => {
                self.dir_leave();
                if w.backend.borrow().is_thumbnail() {
                    self.show_files_widget(false);
                }
            }
            gdk::keys::constants::n => {
                if w.eog.zoom_mode() == eog::ZoomMode::Fit {
                    w.eog.set_zoom_mode(eog::ZoomMode::None);
                } else {
                    w.eog.set_zoom_mode(eog::ZoomMode::Fit);
                }
            }
            gdk::keys::constants::m | gdk::keys::constants::KP_0 => {
                let backend = w.backend.borrow();
                if backend.is_thumbnail() {
                    let new_size = match self.thumbnail_size.get() {
                        175 => 140,
                        140 => 100,
                        100 => 80,
                        80 => 250,
                        _ => 175,
                    };
                    self.thumbnail_size.set(new_size);
                    let (parent_backend, _selection) = backend.leave();
                    drop(backend);
                    w.backend.replace(parent_backend);
                    let display_size = self.obj().display_size();
                    let thumbnail =
                        Thumbnail::new(display_size.width(), display_size.height(), 0, new_size);
                    let startpage = thumbnail.startpage();
                    let new_backend = <dyn Backend>::thumbnail(thumbnail);
                    self.set_backend(new_backend, startpage, true);
                } else if w.eog.zoom_mode() == eog::ZoomMode::Max {
                    w.eog.set_zoom_mode(eog::ZoomMode::Fill);
                } else {
                    w.eog.set_zoom_mode(eog::ZoomMode::Max);
                }
            }
            gdk::keys::constants::minus | gdk::keys::constants::KP_Subtract => {
                w.file_list_view.set_unsorted();
                if let Some(current) = w.file_list_view.current() {
                    if w.backend.borrow().favorite(&current, Direction::Down) {
                        w.file_list_view.navigate(Direction::Down, Filter::Image, 1);
                    }
                }
            }
            gdk::keys::constants::equal | gdk::keys::constants::KP_Add => {
                w.file_list_view.set_unsorted();
                if let Some(current) = w.file_list_view.current() {
                    if w.backend.borrow().favorite(&current, Direction::Up) {
                        w.file_list_view.navigate(Direction::Down, Filter::Image, 1);
                    }
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
            gdk::keys::constants::_1 => {
                if let Some(current) = w.file_list_view.current() {
                    current.set_sort_column(SortColumn::Index(0));
                    w.file_list_view.goto(&Selection::None);
                }
            }
            gdk::keys::constants::_2 => {
                if !w.backend.borrow().is_thumbnail() {
                    if let Some(current) = w.file_list_view.current() {
                        current.set_sort_column(SortColumn::Index(2));
                        w.file_list_view.goto(&Selection::None);
                    }
                }
            }
            gdk::keys::constants::_3 => {
                if !w.backend.borrow().is_thumbnail() {
                    if let Some(current) = w.file_list_view.current() {
                        current.set_sort_column(SortColumn::Index(3));
                        w.file_list_view.goto(&Selection::None);
                    }
                }
            }
            gdk::keys::constants::_4 => {
                if !w.backend.borrow().is_thumbnail() {
                    if let Some(current) = w.file_list_view.current() {
                        current.set_sort_column(SortColumn::Index(4));
                        w.file_list_view.goto(&Selection::None);
                    }
                }
            }
            _ => (),
        }
    }
}
