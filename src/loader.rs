use crate::{category::Category, draw::draw, error::MviewResult};
use eog::Image;
use gdk_pixbuf::Pixbuf;
use gio::{prelude::FileExt, Cancellable, File, MemoryInputStream};
use glib::{Bytes, ObjectExt};
use image::{
    codecs::gif::GifDecoder, io::Reader, AnimationDecoder, GenericImageView, ImageDecoder,
};
use std::{fs, path::Path};

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
        match Self::image_from_memory_gtk(buf) {
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

    pub fn image_from_memory_gtk(buf: Vec<u8>) -> MviewResult<Image> {
        let bytes = Bytes::from_owned(buf);
        let stream = MemoryInputStream::from_bytes(&bytes);
        let image = Image::new_stream(&stream)?;
        Ok(image)
    }

    pub fn image_from_file_image_rs(filename: &str) -> MviewResult<Image> {
        println!("image2");

        let x = Reader::open(filename)?;
        let x = x.with_guessed_format()?;

        if x.format() == Some(image::ImageFormat::Gif) {
            println!("Is gif");
            // let dec =x.into_decoder().unwrap();
            let dec = GifDecoder::new(x.into_inner())?;
            println!("Dimensions: {:?}", dec.dimensions());
            let x = dec.into_frames();

            // println!("LEN {}", x.count());

            for frame in x {
                println!("FRAME");
                let x = frame?;
                println!("Delay {:?}", x.delay());
                let _y = x.into_buffer();
            }

            return Ok(draw("gif").unwrap());
        }

        // let d = x.unwrap().into_decoder().unwrap();

        // match image::open(Path::new(&filename)) {

        let im = x.decode()?;

        // The dimensions method returns the images width and height
        println!(
            "Dynamic image: dimensions={:?}, colortype={:?}",
            im.dimensions(),
            im.color()
        );
        let (width, height) = im.dimensions();
        let b = im.as_bytes();
        match im {
            image::DynamicImage::ImageRgb8(_) => {
                let data = Bytes::from(b);
                let colorspace = gdk_pixbuf::Colorspace::Rgb;
                let has_alpha = false;
                let bits_per_sample = 8;
                let rowstride = 3 * width;
                let pixbuf = Pixbuf::from_bytes(
                    &data,
                    colorspace,
                    has_alpha,
                    bits_per_sample,
                    width as i32,
                    height as i32,
                    rowstride as i32,
                );
                println!("ImageRgb8 {}", pixbuf.width());
                Ok(Image::new_pixbuf(&pixbuf))
            }
            image::DynamicImage::ImageRgba8(_) => {
                let data = Bytes::from(b);
                let colorspace = gdk_pixbuf::Colorspace::Rgb;
                let has_alpha = true;
                let bits_per_sample = 8;
                let rowstride = 4 * width;
                let pixbuf = Pixbuf::from_bytes(
                    &data,
                    colorspace,
                    has_alpha,
                    bits_per_sample,
                    width as i32,
                    height as i32,
                    rowstride as i32,
                );
                println!("ImageRgb8a {}", pixbuf.width());
                Ok(Image::new_pixbuf(&pixbuf))
            }
            // image::DynamicImage::ImageLuma8(_) => todo!(),
            // image::DynamicImage::ImageLumaA8(_) => todo!(),
            // image::DynamicImage::ImageLuma16(_) => todo!(),
            // image::DynamicImage::ImageLumaA16(_) => todo!(),
            // image::DynamicImage::ImageRgb16(_) => todo!(),
            // image::DynamicImage::ImageRgba16(_) => todo!(),
            // image::DynamicImage::ImageRgb32F(_) => todo!(),
            // image::DynamicImage::ImageRgba32F(_) => todo!(),
            _ => {
                println!("Strange stuff");
                Ok(draw(&format!("Strange stuff {:?}", im.color())).unwrap())
            }
        }
    }
}
