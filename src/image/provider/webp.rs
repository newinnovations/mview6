use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Seek},
};

use gdk_pixbuf::Pixbuf;
use glib::Bytes;
use image::{buffer::ConvertBuffer, RgbImage, RgbaImage};
use image_webp::WebPDecoder;

use crate::{
    error::MviewResult,
    image::{
        animation::{AnimationFrame, WebPAnimation},
        view::ZoomMode,
        Image,
    },
};

pub struct WebP {}

impl WebP {
    pub fn image_from_file(reader: BufReader<File>) -> MviewResult<Image> {
        let mut decoder = WebPDecoder::new(reader)?;
        dbg!(decoder.num_frames());
        if decoder.is_animated() {
            let (pixbuf, delay_ms) = Self::read_frame(&mut decoder)?;
            Ok(Image::new_file_animation(
                WebPAnimation::<BufReader<File>> {
                    decoder,
                    index: 0,
                    first_run: true,
                    frames: vec![AnimationFrame { delay_ms, pixbuf }],
                },
                ZoomMode::NotSpecified,
            ))
        } else {
            let pixbuf = Self::read_image(&mut decoder)?;
            Ok(Image::new_pixbuf(pixbuf, ZoomMode::NotSpecified))
        }
    }

    pub fn image_from_memory(reader: Cursor<Vec<u8>>) -> MviewResult<Image> {
        let mut decoder = WebPDecoder::new(reader)?;
        dbg!(decoder.num_frames());
        if decoder.is_animated() {
            let (pixbuf, delay_ms) = Self::read_frame(&mut decoder)?;
            Ok(Image::new_memory_animation(
                WebPAnimation::<Cursor<Vec<u8>>> {
                    decoder,
                    index: 0,
                    first_run: true,
                    frames: vec![AnimationFrame { delay_ms, pixbuf }],
                },
                ZoomMode::NotSpecified,
            ))
        } else {
            let pixbuf = Self::read_image(&mut decoder)?;
            Ok(Image::new_pixbuf(pixbuf, ZoomMode::NotSpecified))
        }
    }

    pub fn read_image<T: BufRead + Seek>(decoder: &mut WebPDecoder<T>) -> MviewResult<Pixbuf> {
        let (width, height) = decoder.dimensions();
        let img = if decoder.has_alpha() {
            let mut img = RgbaImage::new(width, height);
            decoder.read_image(&mut img)?;
            img
        } else {
            let mut img = RgbImage::new(width, height);
            decoder.read_image(&mut img)?;
            img.convert()
        };
        let pixbuf = Pixbuf::from_bytes(
            &Bytes::from(img.as_raw()),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            img.width() as i32,
            img.height() as i32,
            (img.width() * 4) as i32,
        );
        Ok(pixbuf)
    }

    pub fn read_frame<T: BufRead + Seek>(
        decoder: &mut WebPDecoder<T>,
    ) -> MviewResult<(Pixbuf, u32)> {
        let (width, height) = decoder.dimensions();
        let (img, delay) = if decoder.has_alpha() {
            let mut img = RgbaImage::new(width, height);
            let delay = decoder.read_frame(&mut img)?;
            (img, delay)
        } else {
            let mut img = RgbImage::new(width, height);
            let delay = decoder.read_frame(&mut img)?;
            (img.convert(), delay)
        };
        let pixbuf = Pixbuf::from_bytes(
            &Bytes::from(img.as_raw()),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            img.width() as i32,
            img.height() as i32,
            (img.width() * 4) as i32,
        );
        Ok((pixbuf, delay))
    }
}
