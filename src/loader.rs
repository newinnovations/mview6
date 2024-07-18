use crate::{
    category::Category,
    draw::draw,
    error::{AppError, MviewError, MviewResult},
};
use eog::Image;
use gdk_pixbuf::Pixbuf;
use gio::{prelude::FileExt, Cancellable, File, MemoryInputStream};
use glib::{Bytes, ObjectExt};
use image::{io::Reader, DynamicImage, GenericImageView};
use std::{fs, io::Cursor, path::Path};

pub struct Loader {}

impl Loader {
    pub fn image_from_file(filename: &str) -> Image {
        if let Ok(im) = Self::image_from_file_gtk(filename) {
            return im;
        }
        match Self::image_from_file_image_rs(filename) {
            Ok(im) => im,
            Err(e) => draw(&format!("Error: {:?}", e)).unwrap(),
        }
    }

    pub fn image_from_memory(buf: Vec<u8>) -> Image {
        if let Ok(im) = Self::image_from_memory_gtk(&buf) {
            return im;
        }
        match Self::image_from_memory_image_rs(&buf) {
            Ok(im) => im,
            Err(e) => draw(&format!("Error: {:?}", e)).unwrap(),
        }
    }

    pub fn image_from_file_gtk(filename: &str) -> MviewResult<Image> {
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
                let d = draw(name);
                return d;
            }
            _ => (),
        };

        let file = File::for_parse_name(filename);
        let stream = file.read(Cancellable::NONE)?;
        let image = Image::new_stream(&stream)?;

        let filename_c = filename.to_string();
        image.add_weak_ref_notify(move || {
            println!("**image [{filename_c}] disposed**");
        });

        Ok(image)
    }

    pub fn image_from_memory_gtk(buf: &Vec<u8>) -> MviewResult<Image> {
        let bytes = Bytes::from(buf);
        let stream = MemoryInputStream::from_bytes(&bytes);
        let image = Image::new_stream(&stream)?;
        Ok(image)
    }

    pub fn image_from_memory_image_rs(buf: &Vec<u8>) -> MviewResult<Image> {
        let pixbuf = Self::pixbuf_from_memory(buf)?;
        Ok(Image::new_pixbuf(&pixbuf))
    }

    pub fn image_from_file_image_rs(filename: &str) -> MviewResult<Image> {
        let pixbuf = Self::pixbuf_from_file(filename)?;
        Ok(Image::new_pixbuf(&pixbuf))
    }

    pub fn pixbuf_from_memory(buf: &Vec<u8>) -> MviewResult<Pixbuf> {
        let reader = Reader::new(Cursor::new(buf));
        let reader = reader.with_guessed_format()?;
        let dynamic_image = reader.decode()?;
        Self::image_rs_to_pixbuf(dynamic_image)
    }

    pub fn pixbuf_from_file(filename: &str) -> MviewResult<Pixbuf> {
        let reader = Reader::open(filename)?;
        let reader = reader.with_guessed_format()?;
        let dynamic_image = reader.decode()?;
        Self::image_rs_to_pixbuf(dynamic_image)
    }

    pub fn dynimg_from_file(filename: &str) -> MviewResult<DynamicImage> {
        let reader = Reader::open(filename)?;
        let reader = reader.with_guessed_format()?;
        Ok(reader.decode()?)
    }

    pub fn image_rs_to_pixbuf(im: DynamicImage) -> MviewResult<Pixbuf> {
        let (width, height) = im.dimensions();
        let colorspace;
        let has_alpha;
        let bits_per_sample;
        let rowstride;
        match im {
            image::DynamicImage::ImageRgb8(_) => {
                colorspace = gdk_pixbuf::Colorspace::Rgb;
                has_alpha = false;
                bits_per_sample = 8;
                rowstride = 3 * width;
            }
            image::DynamicImage::ImageRgba8(_) => {
                colorspace = gdk_pixbuf::Colorspace::Rgb;
                has_alpha = true;
                bits_per_sample = 8;
                rowstride = 4 * width;
            }
            _ => {
                return Err(MviewError::App(AppError::new(&format!(
                    "Unsupported color space {:?}",
                    im.color()
                ))));
            }
        }
        // println!(
        //     "Image.rs {:?} {width}x{height} alpha={has_alpha}",
        //     im.color()
        // );
        let pixbuf = Pixbuf::from_bytes(
            &Bytes::from(im.as_bytes()),
            colorspace,
            has_alpha,
            bits_per_sample,
            width as i32,
            height as i32,
            rowstride as i32,
        );
        Ok(pixbuf)
    }
}
