use std::{fs, path::Path, time::SystemTime};

use crate::{
    category::Category,
    error::MviewResult,
    image::{animation::Animation, draw::draw, Image},
};
use gdk::prelude::{PixbufAnimationExt, PixbufAnimationExtManual, PixbufLoaderExt};
use gdk_pixbuf::PixbufLoader;
use gio::{
    prelude::{FileExt, InputStreamExt},
    Cancellable, File, MemoryInputStream,
};
use glib::{Bytes, IsA};

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
        Self::image_from_stream(&stream)
    }

    pub fn image_from_memory(buf: &Vec<u8>) -> MviewResult<Image> {
        let bytes = Bytes::from(buf);
        let stream = MemoryInputStream::from_bytes(&bytes);
        Self::image_from_stream(&stream)
    }

    pub fn image_from_stream(stream: &impl IsA<gio::InputStream>) -> MviewResult<Image> {
        let cancellable = Option::<Cancellable>::None.as_ref();
        let loader = PixbufLoader::new();
        loop {
            let b = stream.read_bytes(65536, cancellable)?;
            if b.len() == 0 {
                break;
            }
            loader.write_bytes(&b)?;
        }
        loader.close()?;
        stream.close(cancellable)?;
        if let Some(animation) = loader.animation() {
            if animation.is_static_image() {
                Ok(Image::new_pixbuf(animation.static_image()))
            } else {
                let iter = animation.iter(Some(SystemTime::now()));
                Ok(Image::new_animation(Animation::Gdk(iter)))
            }
        } else {
            Err("No image data".into())
        }
    }
}
