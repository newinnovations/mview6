use std::{
    cell::RefCell,
    panic, thread,
    time::{self, SystemTime},
};

use super::{
    archive_rar::TRarSource, archive_zip::TZipSource, empty_store, filesystem::TFileSource,
    Backend, Backends, Columns, Selection, TreeModelMviewExt,
};
use crate::{
    backends::{archive_rar::RarArchive, archive_zip::ZipArchive, filesystem::FileSystem},
    category::Category,
    error::MviewResult,
    image::ImageLoader,
    window::MViewWidgets,
};
use eog::{Image, ImageExt, ScrollView, ScrollViewExt};
use gdk_pixbuf::Pixbuf;
use gtk::{
    prelude::{GtkListStoreExtManual, TreeModelExt},
    ListStore, TreeIter,
};
use image::DynamicImage;

#[derive(Debug)]
pub struct Thumbnail {
    size: i32,
    sheet_x: i32,
    sheet_y: i32,
    separator_x: i32,
    separator_y: i32,
    parent: RefCell<Box<dyn Backend>>,
    parent_pos: i32,
}

impl Default for Thumbnail {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Thumbnail {
    pub fn new(pos: i32) -> Self {
        Thumbnail {
            size: 175,
            sheet_x: 3840, // 1920,
            sheet_y: 2160, // 1080,
            separator_x: 4,
            separator_y: 4,
            parent: RefCell::new(<dyn Backend>::invalid()),
            parent_pos: pos,
        }
    }

    pub fn capacity_x(&self) -> i32 {
        (self.sheet_x + self.separator_x) / (self.size + self.separator_x)
    }
    pub fn capacity_y(&self) -> i32 {
        (self.sheet_y + self.separator_y) / (self.size + self.separator_y)
    }

    pub fn capacity(&self) -> i32 {
        self.capacity_x() * self.capacity_y()
    }

    pub fn startpage(&self) -> Selection {
        Selection::Index(self.parent_pos as u32 / self.capacity() as u32)
    }

    // pub fn sheet(&self) -> MviewResult<Image> {
    //     let backend = self.parent.borrow();
    //     let store = backend.store();
    //     // let caption = "sheet 1 of 100";
    //     let offset_x = (self.sheet_x - self.capacity_col() * (self.size + self.separator_x)
    //         + self.separator_x)
    //         / 2;
    //     let offset_y = (self.sheet_y - self.capacity_row() * (self.size + self.separator_y)
    //         + self.separator_y)
    //         / 2;

    //     let surface = ImageSurface::create(Format::ARgb32, self.sheet_x, self.sheet_y)?;
    //     let context = Context::new(&surface)?;

    //     context.set_source_rgb(1.0, 0.2, 0.4);
    //     // context.set_source_rgb(0.0, 0.0, 0.0);
    //     context.paint()?;

    //     // // graphics.setBackground(Color.white);
    //     // // graphics.setFont(new Font("LucidaSans", Font.BOLD, 10));

    //     // context.select_font_face(
    //     //     "LucidaSans",
    //     //     cairo::FontSlant::Normal,
    //     //     cairo::FontWeight::Bold,
    //     // ); //Bold);
    //     // let extends = context.text_extents(&caption)?;
    //     // dbg!(&extends);

    //     // // FontMetrics fm = graphics.getFontMetrics();
    //     // // int txtW = fm.stringWidth(caption);
    //     // // int txtH = fm.getHeight();
    //     // let txtW = extends.width();
    //     // let txtH = extends.height();
    //     // // graphics.clearRect(sheetX - txtW - 15, 5, txtW + 10, txtH + 7);
    //     // // graphics.setPaint(Color.black);
    //     // // graphics.drawString(caption, sheetX - txtW - 10, txtH + 5);

    //     let mut done = false;
    //     if let Some(iter) = store.iter_nth_child(None, 0) {
    //         for y in 0..self.capacity_row() {
    //             if done {
    //                 break;
    //             }
    //             for x in 0..self.capacity_col() {
    //                 let f = store.filename(&iter);
    //                 dbg!(&f);

    //                 if let Ok(thumb) = backend.thumb(&store, &iter) {
    //                     let thumb = pixbuf_scale(thumb, self.size);
    //                     let tox = (self.size - thumb.width()) / 2;
    //                     let toy = (self.size - thumb.height()) / 2;
    //                     let topleft_x = offset_x + tox + x * (self.size + self.separator_x);
    //                     let topleft_y = offset_y + toy + y * (self.size + self.separator_y);
    //                     context.set_source_pixbuf(&thumb, topleft_x as f64, topleft_y as f64);
    //                     context.paint()?;
    //                 }

    //                 if !store.iter_next(&iter) {
    //                     done = true;
    //                     break;
    //                 }
    //             }
    //         }
    //     }

    //     let image = Image::new_image_surface(&surface);
    //     image.set_zoom_mode(eog::ZoomMode::None);
    //     dbg!(image.pixbuf());

    //     Ok(image)
    // }

    pub fn offset(&self) -> (i32, i32) {
        (
            (self.sheet_x - self.capacity_x() * (self.size + self.separator_x) + self.separator_x)
                / 2,
            (self.sheet_y - self.capacity_y() * (self.size + self.separator_y) + self.separator_y)
                / 2,
        )
    }

    pub fn sheet(&self, page: i32) -> Vec<TTask> {
        let backend = self.parent.borrow();
        let store = backend.store();
        let (offset_x, offset_y) = self.offset();

        // let caption = "sheet 1 of 100";

        let mut res = Vec::<TTask>::new();

        let mut done = false;
        let mut tid = 0;
        if let Some(iter) = store.iter_nth_child(None, page * self.capacity()) {
            for row in 0..self.capacity_y() {
                if done {
                    break;
                }
                for col in 0..self.capacity_x() {
                    let source = backend.thumb(&store, &iter);
                    if !matches!(source, TSource::None) {
                        let task = TTask::new(
                            tid,
                            self.size as u32,
                            offset_x + col * (self.size + self.separator_x),
                            offset_y + row * (self.size + self.separator_y),
                            source,
                        );
                        res.push(task);
                        tid += 1;
                    }
                    if !store.iter_next(&iter) {
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

        // let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        // let modified = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
        // let file_size = metadata.len();
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
                    // (Columns::Folder as u32, &entry.folder),
                    // (Columns::Size as u32, &file_size),
                    // (Columns::Modified as u32, &modified),
                ],
            );
        }
        store
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        (self.parent.borrow().backend().dynbox(), Selection::None)
    }

    fn image(&self, w: &MViewWidgets, model: &ListStore, iter: &TreeIter) -> Image {
        let pixbuf = Pixbuf::new(
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            self.sheet_x,
            self.sheet_y,
        )
        .unwrap();

        pixbuf.fill(0x202020ff);

        let image = Image::new_pixbuf(&pixbuf);
        image.set_zoom_mode(eog::ZoomMode::None);
        let id = image.id();
        let page = model.index(iter);
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

    fn click(
        &self,
        model: &ListStore,
        iter: &TreeIter,
        x: f64,
        y: f64,
    ) -> Option<(Box<dyn Backend>, Selection)> {
        let (offset_x, offset_y) = self.offset();

        // dbg!(x, y, offset_x, offset_y);

        let x = (x as i32 - offset_x) / (self.size + self.separator_x);
        let y = (y as i32 - offset_y) / (self.size + self.separator_y);

        // dbg!(x, y);

        if x < 0 || y < 0 || x >= self.capacity_x() || y >= self.capacity_y() {
            return None;
        }

        let page = model.index(iter) as i32;
        let pos = page * self.capacity() + y * self.capacity_x() + x;
        // dbg!(pos);

        let backend = self.parent.borrow();
        let store = backend.store();
        if let Some(iter) = store.iter_nth_child(None, pos) {
            let source = backend.thumb(&store, &iter);
            match source {
                TSource::FileSource(src) => Some((
                    self.parent.borrow().backend().dynbox(),
                    Selection::Name(src.filename()),
                )),
                TSource::ZipSource(src) => Some((
                    self.parent.borrow().backend().dynbox(),
                    Selection::Index(src.index()),
                )),
                TSource::RarSource(src) => Some((
                    self.parent.borrow().backend().dynbox(),
                    Selection::Name(src.selection()),
                )),
                TSource::None => None,
            }
        } else {
            None
        }
    }
}

fn thumb_result(res: MviewResult<DynamicImage>) -> Option<DynamicImage> {
    match res {
        Ok(image) => Some(image),
        Err(error) => {
            println!("Thumbnail failed: {:?}", error);
            None
        }
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
                    let image = match panic::catch_unwind(|| match &task.source {
                        TSource::FileSource(src) => thumb_result(FileSystem::get_thumbnail(src)),
                        TSource::ZipSource(src) => thumb_result(ZipArchive::get_thumbnail(src)),
                        TSource::RarSource(src) => thumb_result(RarArchive::get_thumbnail(src)),
                        TSource::None => None,
                    }) {
                        Ok(image) => image,
                        Err(_) => {
                            println!("*** Panic in image-rs/zune-jpeg ***");
                            None
                        }
                    };
                    let image = match image {
                        Some(im) => Some(im.resize(
                            task.size,
                            task.size,
                            image::imageops::FilterType::Lanczos3,
                        )),
                        None => None,
                    };

                    let _ = sender_clone.send(Message::Result(TResult::new(id, task, image)));
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
            if let Some(thumb) = result.image {
                // println!("{tid:3}:    -- got thumb image");
                match ImageLoader::image_rs_to_pixbuf(thumb) {
                    Ok(thumb_pb) => {
                        if let Some(image_pb) = image.pixbuf() {
                            let size = result.task.size as i32;
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
            } else {
                // println!("{tid:3}:    -- no thumb image");
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
pub enum TSource {
    FileSource(TFileSource),
    ZipSource(TZipSource),
    RarSource(TRarSource),
    None,
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
    source: TSource,
}

impl TTask {
    pub fn new(tid: i32, size: u32, x: i32, y: i32, source: TSource) -> Self {
        TTask {
            tid,
            size,
            position: (x, y),
            source,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TResult {
    id: i32,
    task: TTask,
    image: Option<DynamicImage>,
}

impl TResult {
    pub fn new(id: i32, task: TTask, image: Option<DynamicImage>) -> Self {
        TResult { id, task, image }
    }
}

pub enum Message {
    Command(TCommand),
    Result(TResult),
}
