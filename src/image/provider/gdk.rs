use std::{fs, path::Path};

use gio::{prelude::FileExt, Cancellable, File, MemoryInputStream};
use glib::Bytes;

use crate::{
    category::Category,
    error::MviewResult,
    image::{draw::draw, view::ZoomMode, Image},
};

pub struct GdkImageLoader {}

impl GdkImageLoader {
    pub fn image_from_file(filename: &str) -> MviewResult<Image> {
        let path = Path::new(&filename);

        let cat = match fs::metadata(path) {
            Ok(metadata) => Category::determine(filename, metadata.is_dir()),
            Err(_) => Category::Unsupported,
        };

        match cat {
            Category::Direcory | Category::Archive | Category::Unsupported => {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                return Ok(draw(name));
            }
            _ => (),
        };

        let file = File::for_parse_name(filename);
        let stream = file.read(Cancellable::NONE)?;
        let image = Image::new_stream(&stream, ZoomMode::NotSpecified)?;
        Ok(image)
    }

    pub fn image_from_memory(buf: &Vec<u8>) -> MviewResult<Image> {
        let bytes = Bytes::from(buf);
        let stream = MemoryInputStream::from_bytes(&bytes);
        let image = Image::new_stream(&stream, ZoomMode::NotSpecified)?;
        Ok(image)
    }
}
