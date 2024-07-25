use std::{
    cell::RefCell,
    panic, thread,
    time::{self, SystemTime},
};

use super::{
    archive_rar::TRarReference, archive_zip::TZipReference, empty_store,
    filesystem::TFileReference, Backend, Backends, Columns, Selection,
};
use crate::{
    backends::{archive_rar::RarArchive, archive_zip::ZipArchive, filesystem::FileSystem},
    category::Category,
    draw::{text_thumb, thumbnail_sheet},
    error::MviewResult,
    filelistview::Cursor,
    image::ImageLoader,
    window::MViewWidgets,
};
use eog::{Image, ImageExt, ScrollView, ScrollViewExt};
use gdk_pixbuf::Pixbuf;
use gtk::{
    prelude::{GtkListStoreExtManual, TreeModelExt},
    ListStore,
};
use image::DynamicImage;

#[derive(Debug)]
pub struct Thumbnail {
    size: i32,
    width: i32,
    height: i32,
    // calculated
    separator_x: i32,
    separator_y: i32,
    capacity_x: i32,
    capacity_y: i32,
    offset_x: i32,
    offset_y: i32,
    // references
    parent: RefCell<Box<dyn Backend>>,
    parent_pos: i32,
}

impl Default for Thumbnail {
    fn default() -> Self {
        Self::new(800, 600, 0, 175)
    }
}

impl Thumbnail {
    pub fn new(width: i32, height: i32, position: i32, size: i32) -> Self {
        let footer = 50;
        let min_separator = 5;

        let capacity_x = (width + min_separator) / (size + min_separator);
        let capacity_y = (height - footer + min_separator) / (size + min_separator);

        let separator_x = (width - capacity_x * size) / capacity_x;
        let separator_y = (height - footer - capacity_y * size) / capacity_y;

        let offset_x = (width - capacity_x * (size + separator_x) + separator_x) / 2;
        let offset_y = (height - footer - capacity_y * (size + separator_y) + separator_y) / 2;

        Thumbnail {
            size,
            width,
            height,
            separator_x,
            separator_y,
            capacity_x,
            capacity_y,
            offset_x,
            offset_y,
            parent: RefCell::new(<dyn Backend>::invalid()),
            parent_pos: position,
        }
    }

    pub fn capacity(&self) -> i32 {
        self.capacity_x * self.capacity_y
    }

    pub fn startpage(&self) -> Selection {
        Selection::Index(self.parent_pos as u32 / self.capacity() as u32)
    }

    pub fn sheet(&self, page: i32) -> Vec<TTask> {
        let backend = self.parent.borrow();
        let store = backend.store();

        let mut res = Vec::<TTask>::new();

        let mut done = false;
        let mut tid = 0;
        let start = page * self.capacity();
        if let Some(iter) = store.iter_nth_child(None, start) {
            let cursor = Cursor::new(store, iter, start);
            for row in 0..self.capacity_y {
                if done {
                    break;
                }
                for col in 0..self.capacity_x {
                    let source = backend.entry(&cursor);
                    if !matches!(source.reference, TReference::None) {
                        let task = TTask::new(
                            tid,
                            self.size as u32,
                            self.offset_x + col * (self.size + self.separator_x),
                            self.offset_y + row * (self.size + self.separator_y),
                            source,
                        );
                        res.push(task);
                        tid += 1;
                    }
                    if !cursor.next() {
                        done = true;
                        break;
                    }
                }
            }
        }

        res
    }
}

impl Backend for Thumbnail {
    fn class_name(&self) -> &str {
        "Thumbnail"
    }

    fn path(&self) -> &str {
        "/thumbnail"
    }

    fn store(&self) -> ListStore {
        let parent_store = self.parent.borrow().store();
        let num_items = parent_store.iter_n_children(None);
        let pages = (1 + num_items / self.capacity()) as u32;
        let store = empty_store();
        let cat = Category::Image;

        for page in 0..pages {
            let name = format!("Thumbnail page {:7}", page + 1);
            store.insert_with_values(
                None,
                &[
                    (Columns::Cat as u32, &cat.id()),
                    (Columns::Icon as u32, &cat.icon()),
                    (Columns::Name as u32, &name),
                    (Columns::Index as u32, &page),
                ],
            );
        }
        store
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        let parent_backend = self.parent.replace(<dyn Backend>::invalid());
        (parent_backend, Selection::None)
    }

    fn image(&self, w: &MViewWidgets, cursor: &Cursor) -> Image {
        let page = cursor.index();
        let caption = format!("sheet {} of {}", page + 1, cursor.store_size());

        let image = match thumbnail_sheet(self.width, self.height, self.offset_x, &caption) {
            Ok(image) => image,
            Err(_) => {
                let pixbuf = Pixbuf::new(
                    gdk_pixbuf::Colorspace::Rgb,
                    true,
                    8,
                    self.width,
                    self.height,
                )
                .unwrap();
                pixbuf.fill(0x202020ff);
                let image = Image::new_pixbuf(&pixbuf);
                image.set_zoom_mode(eog::ZoomMode::None);
                image
            }
        };
        let id = image.id();
        let command = TCommand::new(id, self.sheet(page as i32));

        let _ = w.sender.send(Message::Command(command));

        image
    }

    fn set_parent(&self, parent: Box<dyn Backend>) {
        self.parent.replace(parent);
    }

    fn is_thumbnail(&self) -> bool {
        true
    }

    fn backend(&self) -> Backends {
        self.parent.borrow().backend()
    }

    fn click(&self, current: &Cursor, x: f64, y: f64) -> Option<(Box<dyn Backend>, Selection)> {
        let x = (x as i32 - self.offset_x) / (self.size + self.separator_x);
        let y = (y as i32 - self.offset_y) / (self.size + self.separator_y);

        if x < 0 || y < 0 || x >= self.capacity_x || y >= self.capacity_y {
            return None;
        }

        let page = current.index() as i32;
        let pos = page * self.capacity() + y * self.capacity_x + x;

        let backend = self.parent.borrow();
        let store = backend.store();
        if let Some(iter) = store.iter_nth_child(None, pos) {
            let cursor = Cursor::new(store, iter, pos);
            let source = backend.entry(&cursor).reference;
            match source {
                TReference::FileReference(src) => Some((
                    self.parent.borrow().backend().dynbox(),
                    Selection::Name(src.filename()),
                )),
                TReference::ZipReference(src) => Some((
                    self.parent.borrow().backend().dynbox(),
                    Selection::Index(src.index()),
                )),
                TReference::RarReference(src) => Some((
                    self.parent.borrow().backend().dynbox(),
                    Selection::Name(src.selection()),
                )),
                TReference::None => None,
            }
        } else {
            None
        }
    }
}

fn thumb_result(res: MviewResult<DynamicImage>, task: &TTask) -> TResultOption {
    match res {
        Ok(image) => {
            let image = image.resize(task.size, task.size, image::imageops::FilterType::Lanczos3);
            TResultOption::Image(image)
        }
        Err(_error) => match task.source.category {
            Category::Direcory => {
                TResultOption::Message(TMessage::new("directory", &task.source.name))
            }
            Category::Archive => {
                TResultOption::Message(TMessage::new("archive", &task.source.name))
            }
            Category::Unsupported => {
                TResultOption::Message(TMessage::new("unsupported", &task.source.name))
            }
            _ => TResultOption::Message(TMessage::new("error", &task.source.name)),
        },
    }
}

pub fn start_thumbnail_task(
    sender: &glib::Sender<Message>,
    eog: &ScrollView,
    command: &TCommand,
    current_task: &mut usize,
) {
    // let elapsed = command.elapsed();
    // println!("ThumbnailTask: {:7.3}", elapsed);
    if let Some(image) = eog.image() {
        let id = image.id();
        if command.id == id {
            // println!("-- command id is ok: {id}");
            let sender_clone = sender.clone();
            if let Some(task) = command.tasks.get(*current_task) {
                *current_task += 1;
                let task = task.clone();
                // let tid = task.tid;
                thread::spawn(move || {
                    // println!("{tid:3}: start {:7.3}", elapsed);
                    // thread::sleep(time::Duration::from_secs(2));
                    thread::sleep(time::Duration::from_millis(1));
                    let result = match panic::catch_unwind(|| match &task.source.reference {
                        TReference::FileReference(src) => {
                            thumb_result(FileSystem::get_thumbnail(src), &task)
                        }
                        TReference::ZipReference(src) => {
                            thumb_result(ZipArchive::get_thumbnail(src), &task)
                        }
                        TReference::RarReference(src) => {
                            thumb_result(RarArchive::get_thumbnail(src), &task)
                        }
                        TReference::None => {
                            TResultOption::Message(TMessage::new("none", "TEntry::None"))
                        }
                    }) {
                        Ok(image) => image,
                        Err(_) => TResultOption::Message(TMessage::new("panic", &task.source.name)),
                    };
                    let _ = sender_clone.send(Message::Result(TResult::new(id, task, result)));
                });
            }
        } else {
            // println!("-- command id mismatch {} != {id}", command.id);
        }
    }
}

pub fn handle_thumbnail_result(eog: &ScrollView, command: &mut TCommand, result: TResult) -> bool {
    if command.id != result.id {
        return false;
    }
    // let tid = result.task.tid;
    let elapsed = command.elapsed();
    command.todo -= 1;
    // println!("{tid:3}: ready {:7.3} todo={}", elapsed, command.todo);
    if let Some(image) = eog.image() {
        let id = image.id();
        if result.id == id {
            // println!("{tid:3}: -- result id is ok: {id}");

            let pixbuf = match result.result {
                TResultOption::Image(image) => ImageLoader::image_rs_to_pixbuf(image),
                TResultOption::Message(message) => text_thumb(message),
            };

            match pixbuf {
                Ok(thumb_pb) => {
                    if let Some(image_pb) = image.pixbuf() {
                        let size = result.task.size as i32;

                        let thumb_pb = if thumb_pb.width() > size || thumb_pb.height() > size {
                            ImageLoader::pixbuf_scale(thumb_pb, size).unwrap()
                        } else {
                            thumb_pb
                        };

                        let (x, y) = result.task.position;
                        thumb_pb.copy_area(
                            0,
                            0,
                            thumb_pb.width(),
                            thumb_pb.height(),
                            &image_pb,
                            x + (size - thumb_pb.width()) / 2,
                            y + (size - thumb_pb.height()) / 2,
                        );
                    }
                }
                Err(error) => {
                    println!("Thumbnail: failed to convert to pixbuf {:?}", error);
                }
            }
            if command.todo == 0 || (elapsed - command.last_update) > 0.3 {
                if command.last_update == 0.0 {
                    eog.set_image_post();
                }
                image.modified();
                command.last_update = elapsed;
            }
            return command.todo != 0;
        } else {
            // println!("{tid:3}: -- command id mismatch {} != {id}", result.id);
        }
    }
    false
}

#[derive(Debug, Clone)]
pub enum TReference {
    FileReference(TFileReference),
    ZipReference(TZipReference),
    RarReference(TRarReference),
    None,
}

#[derive(Debug, Clone)]
pub struct TEntry {
    category: Category,
    name: String,
    reference: TReference,
}

impl TEntry {
    pub fn new(category: Category, name: &str, reference: TReference) -> Self {
        TEntry {
            category,
            name: name.to_string(),
            reference,
        }
    }
}

impl Default for TEntry {
    fn default() -> Self {
        Self {
            category: Category::Unsupported,
            name: Default::default(),
            reference: TReference::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TCommand {
    id: i32,
    start: SystemTime,
    tasks: Vec<TTask>,
    todo: usize,
    last_update: f64,
}

impl Default for TCommand {
    fn default() -> Self {
        Self {
            id: Default::default(),
            start: SystemTime::now(),
            tasks: Default::default(),
            todo: 0,
            last_update: 0.0,
        }
    }
}

impl TCommand {
    pub fn new(id: i32, tasks: Vec<TTask>) -> Self {
        let todo = tasks.len();
        TCommand {
            id,
            start: SystemTime::now(),
            tasks,
            todo,
            last_update: 0.0,
        }
    }

    pub fn elapsed(&self) -> f64 {
        if let Ok(elapsed) = self.start.elapsed() {
            elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
        } else {
            0.0
        }
    }

    pub fn needs_work(&self) -> bool {
        self.todo != 0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TTask {
    tid: i32,
    size: u32,
    position: (i32, i32),
    source: TEntry,
}

impl TTask {
    pub fn new(tid: i32, size: u32, x: i32, y: i32, source: TEntry) -> Self {
        TTask {
            tid,
            size,
            position: (x, y),
            source,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TMessage {
    title: String,
    message: String,
}

impl TMessage {
    pub fn new(title: &str, message: &str) -> Self {
        TMessage {
            title: title.to_string(),
            message: message.to_string(),
        }
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub enum TResultOption {
    Image(DynamicImage),
    Message(TMessage),
}

#[derive(Debug, Clone)]
pub struct TResult {
    id: i32,
    task: TTask,
    result: TResultOption,
}

impl TResult {
    pub fn new(id: i32, task: TTask, result: TResultOption) -> Self {
        TResult { id, task, result }
    }
}

pub enum Message {
    Command(TCommand),
    Result(TResult),
}
