use std::{
    cmp::min,
    io::{BufRead, Seek},
    time::SystemTime,
};

use crate::{
    error::MviewResult,
    image::{animation::Animation, provider::ExifReader, Image},
};
use gdk_pixbuf::PixbufLoader;
use gtk4::prelude::{PixbufAnimationExt, PixbufAnimationExtManual, PixbufLoaderExt};

pub struct GdkImageLoader {}

impl GdkImageLoader {
    pub fn image_from_reader<T: BufRead + Seek>(reader: &mut T) -> MviewResult<Image> {
        let mut buf = [0u8; 65536];
        let loader = PixbufLoader::new();
        loop {
            let num_read = reader.read(&mut buf)?;
            if num_read == 0 {
                break;
            }
            let num_read = min(num_read, buf.len());
            loader.write(&buf[0..num_read])?;
        }
        loader.close()?;
        if let Some(animation) = loader.animation() {
            if animation.is_static_image() {
                Ok(Image::new_pixbuf(animation.static_image(), reader.exif()))
            } else {
                let iter = animation.iter(Some(SystemTime::now()));
                Ok(Image::new_animation(Animation::Gdk(iter)))
            }
        } else {
            Err("No image data".into())
        }
    }
}
