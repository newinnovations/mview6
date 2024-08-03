use std::{panic, thread, time};

use image::DynamicImage;

use crate::{
    backends::{archive_rar::RarArchive, archive_zip::ZipArchive, filesystem::FileSystem},
    category::Category,
    error::MviewResult,
    image::{draw::text_thumb, provider::image_rs::RsImageLoader, view::ImageView},
};

use super::{Message, TCommand, TMessage, TReference, TResult, TResultOption, TTask};

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
    image_view: &ImageView,
    command: &TCommand,
    current_task: &mut usize,
) {
    // let elapsed = command.elapsed();
    // println!("ThumbnailTask: {:7.3}", elapsed);
    let id = image_view.image_id();
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

pub fn handle_thumbnail_result(
    image_view: &ImageView,
    command: &mut TCommand,
    result: TResult,
) -> bool {
    if command.id != result.id {
        return false;
    }
    // let tid = result.task.tid;
    let elapsed = command.elapsed();
    command.todo -= 1;
    // println!("{tid:3}: ready {:7.3} todo={}", elapsed, command.todo);
    if result.id == image_view.image_id() {
        // println!("{tid:3}: -- result id is ok: {id}");

        let pixbuf = match result.result {
            TResultOption::Image(image) => RsImageLoader::dynimg_to_pixbuf(image),
            TResultOption::Message(message) => text_thumb(message),
        };

        match pixbuf {
            Ok(thumb_pb) => {
                let size = result.task.size as i32;

                let thumb_pb = if thumb_pb.width() > size || thumb_pb.height() > size {
                    RsImageLoader::pixbuf_scale(thumb_pb, size)
                } else {
                    Some(thumb_pb)
                };

                if let Some(thumb_pb) = thumb_pb {
                    let (x, y) = result.task.position;
                    image_view.draw_pixbuf(
                        &thumb_pb,
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
                image_view.set_image_post();
            }
            image_view.image_modified();
            command.last_update = elapsed;
        }
        return command.todo != 0;
    } else {
        // println!("{tid:3}: -- command id mismatch {} != {id}", result.id);
    }
    false
}
