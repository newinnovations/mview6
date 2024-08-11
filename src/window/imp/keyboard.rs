use super::MViewWindowImp;

use gtk4::{gdk::Key, prelude::*, subclass::prelude::*, SortColumn};

use crate::{
    backends::{thumbnail::Thumbnail, Backend},
    filelistview::{Direction, Filter, Selection, Sort},
    image::view::ZoomMode,
};

impl MViewWindowImp {
    pub(super) fn on_key_press(&self, e: Key) {
        let w = self.widgets();
        match e {
            Key::q => {
                self.obj().close();
            }
            Key::d => {
                self.show_files_widget(true);
                if !self.backend.borrow().is_bookmarks() {
                    self.set_backend(<dyn Backend>::bookmarks(), Selection::None, true);
                }
            }
            Key::i => {
                self.show_info_widget(!w.info_widget.is_visible());
            }
            Key::t => {
                if self.backend.borrow().is_container() {
                    let position = if let Some(cursor) = w.file_list_view.current() {
                        cursor.position()
                    } else {
                        0
                    };
                    if let Some(thumbnail) = Thumbnail::new(
                        w.image_view.allocation(),
                        position,
                        self.thumbnail_size.get(),
                    ) {
                        let startpage = thumbnail.startpage();
                        let new_backend = <dyn Backend>::thumbnail(thumbnail);
                        new_backend.set_sort(&Sort::sort_on_category());
                        self.set_backend(new_backend, startpage, true);
                        // self.show_files_widget(false);
                        // self.obj().fullscreen();
                        // self.full_screen.set(true);
                        // w.image_view.update_mouse_position();
                    }
                }
            }
            Key::w | Key::KP_7 | Key::KP_Home => {
                self.hop(Direction::Up);
            }
            Key::e | Key::KP_9 | Key::KP_Page_Up => {
                self.hop(Direction::Down);
            }
            Key::space | Key::KP_Divide => {
                self.show_files_widget(!w.files_widget.is_visible());
            }
            Key::f | Key::KP_Multiply => {
                if self.full_screen.get() {
                    self.obj().unfullscreen();
                    self.full_screen.set(false);
                } else {
                    self.show_files_widget(false);
                    self.obj().fullscreen();
                    self.full_screen.set(true);
                }
                // w.image_view.update_mouse_position();
            }
            Key::Escape => {
                self.obj().unfullscreen();
                self.full_screen.set(false);
                // w.image_view.update_mouse_position();
            }
            Key::r => {
                w.image_view.rotate(270);
            }
            Key::R => {
                w.image_view.rotate(90);
            }
            Key::Return => {
                self.dir_enter(None);
            }
            Key::BackSpace => {
                self.dir_leave();
                if self.backend.borrow().is_thumbnail() {
                    self.show_files_widget(false);
                }
            }
            Key::n => {
                if w.image_view.zoom_mode() == ZoomMode::Fit {
                    w.image_view.set_zoom_mode(ZoomMode::NoZoom);
                } else {
                    w.image_view.set_zoom_mode(ZoomMode::Fit);
                }
            }
            Key::m | Key::KP_0 => {
                let backend = self.backend.borrow();
                if backend.is_thumbnail() {
                    let new_size = match self.thumbnail_size.get() {
                        175 => 140,
                        140 => 100,
                        100 => 80,
                        80 => 250,
                        _ => 175,
                    };
                    self.thumbnail_size.set(new_size);
                    drop(backend);
                    self.update_thumbnail_backend()
                } else if w.image_view.zoom_mode() == ZoomMode::Max {
                    w.image_view.set_zoom_mode(ZoomMode::Fill);
                } else {
                    w.image_view.set_zoom_mode(ZoomMode::Max);
                }
            }
            Key::minus | Key::KP_Subtract => {
                w.file_list_view.set_unsorted();
                if let Some(current) = w.file_list_view.current() {
                    if self.backend.borrow().favorite(&current, Direction::Down) {
                        w.file_list_view.navigate(Direction::Down, Filter::Image, 1);
                    }
                }
            }
            Key::equal | Key::KP_Add => {
                w.file_list_view.set_unsorted();
                if let Some(current) = w.file_list_view.current() {
                    if self.backend.borrow().favorite(&current, Direction::Up) {
                        w.file_list_view.navigate(Direction::Down, Filter::Image, 1);
                    }
                }
            }
            Key::z | Key::Left | Key::KP_4 => {
                w.file_list_view.navigate(Direction::Up, w.filter(), 1);
            }
            Key::x | Key::Right | Key::KP_6 => {
                w.file_list_view.navigate(Direction::Down, w.filter(), 1);
            }
            Key::a => {
                w.file_list_view
                    .navigate(Direction::Up, Filter::Favorite, 1);
            }
            Key::s => {
                w.file_list_view
                    .navigate(Direction::Down, Filter::Favorite, 1);
            }
            Key::Z => {
                w.file_list_view.navigate(Direction::Up, Filter::None, 1);
            }
            Key::X => {
                w.file_list_view.navigate(Direction::Down, Filter::None, 1);
            }
            Key::Up | Key::KP_8 => {
                w.file_list_view.navigate(Direction::Up, w.filter(), 5);
            }
            Key::Down | Key::KP_2 => {
                w.file_list_view.navigate(Direction::Down, w.filter(), 5);
            }
            Key::Page_Up => {
                w.file_list_view.navigate(Direction::Up, w.filter(), 25);
            }
            Key::Page_Down => {
                w.file_list_view.navigate(Direction::Down, w.filter(), 25);
            }
            Key::Home => {
                w.file_list_view.home();
            }
            Key::End => {
                w.file_list_view.end();
            }
            Key::_1 => {
                if let Some(current) = w.file_list_view.current() {
                    current.set_sort_column(SortColumn::Index(0));
                    w.file_list_view.goto(&Selection::None);
                }
            }
            Key::_2 => {
                if !self.backend.borrow().is_thumbnail() {
                    if let Some(current) = w.file_list_view.current() {
                        current.set_sort_column(SortColumn::Index(2));
                        w.file_list_view.goto(&Selection::None);
                    }
                }
            }
            Key::_3 => {
                if !self.backend.borrow().is_thumbnail() {
                    if let Some(current) = w.file_list_view.current() {
                        current.set_sort_column(SortColumn::Index(3));
                        w.file_list_view.goto(&Selection::None);
                    }
                }
            }
            Key::_4 => {
                if !self.backend.borrow().is_thumbnail() {
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
