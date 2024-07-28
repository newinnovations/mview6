use crate::{
    category::Category,
    draw::draw,
    error::{AppError, MviewError, MviewResult},
};
use eog::Image;
use gdk_pixbuf::Pixbuf;
use gio::{prelude::FileExt, Cancellable, File, MemoryInputStream};
use glib::{Bytes, ObjectExt};
use image::{DynamicImage, GenericImageView, ImageReader};
use std::{fs, io::Cursor, path::Path};

pub struct ImageLoader {}

impl ImageLoader {
    pub fn image_from_file(filename: &str) -> Image {
        let image = if let Ok(im) = Self::image_from_file_gtk(filename) {
            im
        } else {
            match Self::image_from_file_image_rs(filename) {
                Ok(im) => im,
                Err(e) => draw(&format!("Error: {:?}", e)).unwrap(),
            }
        };

        // let image = match Self::image_from_file_image_rs(filename) {
        //     Ok(im) => im,
        //     Err(e) => draw(&format!("Error: {:?}", e)).unwrap(),
        // };

        let filename_c = filename.to_string();
        image.add_weak_ref_notify(move || {
            println!("**image [{filename_c}] disposed**");
        });
        image
    }

    pub fn image_from_memory(buf: Vec<u8>) -> Image {
        let image = if let Ok(im) = Self::image_from_memory_gtk(&buf) {
            im
        } else {
            match Self::image_from_memory_image_rs(&buf) {
                Ok(im) => im,
                Err(e) => draw(&format!("Error: {:?}", e)).unwrap(),
            }
        };
        image.add_weak_ref_notify(move || {
            println!("**image (from memory) disposed**");
        });
        image
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
        let reader = ImageReader::new(Cursor::new(buf));
        let reader = reader.with_guessed_format()?;
        let dynamic_image = reader.decode()?;
        Self::image_rs_to_pixbuf(dynamic_image)
    }

    pub fn pixbuf_from_file(filename: &str) -> MviewResult<Pixbuf> {
        let reader = ImageReader::open(filename)?;
        let reader = reader.with_guessed_format()?;
        let dynamic_image = reader.decode()?;
        Self::image_rs_to_pixbuf(dynamic_image)
    }

    pub fn dynimg_from_memory(buf: &Vec<u8>) -> MviewResult<DynamicImage> {
        let reader = ImageReader::new(Cursor::new(buf));
        let reader = reader.with_guessed_format()?;
        Ok(reader.decode()?)
    }

    pub fn dynimg_from_file(filename: &str) -> MviewResult<DynamicImage> {
        let reader = ImageReader::open(filename)?;
        let reader = reader.with_guessed_format()?;
        Ok(reader.decode()?)
    }

    pub fn image_rs_to_pixbuf(image: DynamicImage) -> MviewResult<Pixbuf> {
        let (width, height) = image.dimensions();
        let colorspace;
        let has_alpha;
        let bits_per_sample;
        let rowstride;

        let image = match image.color() {
            image::ColorType::L8 => DynamicImage::from(image.to_rgb8()),
            image::ColorType::La8 => DynamicImage::from(image.to_rgba8()),
            image::ColorType::L16 => DynamicImage::from(image.to_rgb8()),
            image::ColorType::La16 => DynamicImage::from(image.to_rgba8()),
            image::ColorType::Rgb16 => DynamicImage::from(image.to_rgb8()),
            image::ColorType::Rgba16 => DynamicImage::from(image.to_rgba8()),
            image::ColorType::Rgb32F => DynamicImage::from(image.to_rgb8()),
            image::ColorType::Rgba32F => DynamicImage::from(image.to_rgba8()),
            _ => image,
        };

        match image.color() {
            image::ColorType::Rgb8 => {
                colorspace = gdk_pixbuf::Colorspace::Rgb;
                has_alpha = false;
                bits_per_sample = 8;
                rowstride = 3 * width;
            }
            image::ColorType::Rgba8 => {
                colorspace = gdk_pixbuf::Colorspace::Rgb;
                has_alpha = true;
                bits_per_sample = 8;
                rowstride = 4 * width;
            }
            _ => {
                return Err(MviewError::App(AppError::new(&format!(
                    "Unsupported color space {:?}",
                    image.color()
                ))));
            }
        }
        // println!(
        //     "Image.rs {:?} {width}x{height} alpha={has_alpha}",
        //     im.color()
        // );
        let pixbuf = Pixbuf::from_bytes(
            &Bytes::from(image.as_bytes()),
            colorspace,
            has_alpha,
            bits_per_sample,
            width as i32,
            height as i32,
            rowstride as i32,
        );
        Ok(pixbuf)
    }

    pub fn pixbuf_scale(pixbuf: Pixbuf, size: i32) -> Option<Pixbuf> {
        let width = pixbuf.width();
        let height = pixbuf.height();

        let (thumb_width, thumb_height) = if width > height {
            (size, height * size / width)
        } else {
            (width * size / height, size)
        };

        pixbuf.scale_simple(thumb_width, thumb_height, gdk_pixbuf::InterpType::Bilinear)
    }
}

pub struct ImageSaver {}

impl ImageSaver {
    pub fn save_thumbnail(base_directory: &str, filename: &str, image: &DynamicImage) {
        let thumbnail_dir = format!("{}/.mview", base_directory);
        if !Path::new(&thumbnail_dir).exists() {
            if let Err(error) = fs::create_dir(&thumbnail_dir) {
                println!("Failed to create thumbnail directory: {:?}", error);
                return;
            }
        }
        let thumbnail_path = format!("{thumbnail_dir}/{filename}");

        let image = match image.color() {
            image::ColorType::L16 => &DynamicImage::from(image.to_luma8()),
            image::ColorType::La16 => &DynamicImage::from(image.to_luma_alpha8()),
            image::ColorType::Rgb16 => &DynamicImage::from(image.to_rgb8()),
            image::ColorType::Rgba16 => &DynamicImage::from(image.to_rgba8()),
            image::ColorType::Rgb32F => &DynamicImage::from(image.to_rgb8()),
            image::ColorType::Rgba32F => &DynamicImage::from(image.to_rgba8()),
            _ => image,
        };

        let format = match image.color() {
            image::ColorType::L8 => image::ImageFormat::Jpeg,
            image::ColorType::La8 => image::ImageFormat::WebP,
            image::ColorType::Rgb8 => image::ImageFormat::Jpeg,
            image::ColorType::Rgba8 => image::ImageFormat::WebP,
            _ => {
                println!(
                    "Unsupported image colortype when writing thumbnail {:?}",
                    image.color()
                );
                return;
            }
        };

        if let Err(error) = image.save_with_format(thumbnail_path, format) {
            println!("Failed to write thumbnail: {:?}", error);
        }
    }
}
