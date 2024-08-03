use std::io::{BufRead, Cursor, Seek};

use gdk_pixbuf::Pixbuf;
use glib::Bytes;
use image::{DynamicImage, GenericImageView, ImageReader};

use crate::{
    error::{AppError, MviewError, MviewResult},
    image::{view::ZoomMode, Image},
};

use super::webp::WebP;

pub struct RsImageLoader {}

impl RsImageLoader {
    // pub fn image_from_memory(buffer: &Vec<u8>) -> MviewResult<Image> {
    //     Self::image(ImageReader::new(Cursor::new(buffer)))
    // }

    pub fn dynimg_from_memory(buffer: &Vec<u8>) -> MviewResult<DynamicImage> {
        Self::dynimg(ImageReader::new(Cursor::new(buffer)))
    }

    pub fn dynimg_from_file(filename: &str) -> MviewResult<DynamicImage> {
        Self::dynimg(ImageReader::open(filename)?)
    }
}

impl RsImageLoader {
    pub fn image_from_file(filename: &str) -> MviewResult<Image> {
        let reader = ImageReader::open(filename)?;
        let reader = reader.with_guessed_format()?;
        if let Some(format) = reader.format() {
            match format {
                image::ImageFormat::WebP => WebP::image_from_file(reader.into_inner()),
                _ => Self::image(reader),
            }
        } else {
            Ok(Image::default())
        }
    }

    // ImageReader<Cursor<&Vec<u8>>

    pub fn image_from_memory(buffer: Vec<u8>) -> MviewResult<Image> {
        let reader = ImageReader::new(Cursor::new(buffer));
        let reader = reader.with_guessed_format()?;
        if let Some(format) = reader.format() {
            match format {
                image::ImageFormat::WebP => WebP::image_from_memory(reader.into_inner()),
                _ => Self::image(reader),
            }
        } else {
            Ok(Image::default())
        }
    }
}

impl RsImageLoader {
    pub fn image<T: BufRead + Seek>(reader: ImageReader<T>) -> MviewResult<Image> {
        Ok(Image::new_pixbuf(
            Self::pixbuf(reader)?,
            ZoomMode::NotSpecified,
        ))
    }
    pub fn pixbuf<T: BufRead + Seek>(reader: ImageReader<T>) -> MviewResult<Pixbuf> {
        let reader = reader.with_guessed_format()?;
        let dynamic_image = reader.decode()?;
        Self::dynimg_to_pixbuf(dynamic_image)
    }

    pub fn dynimg<T: BufRead + Seek>(reader: ImageReader<T>) -> MviewResult<DynamicImage> {
        let reader = reader.with_guessed_format()?;
        Ok(reader.decode()?)
    }
}

impl RsImageLoader {
    pub fn dynimg_to_pixbuf(image: DynamicImage) -> MviewResult<Pixbuf> {
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
