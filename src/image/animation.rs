use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Seek},
    time::{Duration, SystemTime},
};

use gdk_pixbuf::{Pixbuf, PixbufAnimationIter};
use image_webp::WebPDecoder;

use super::{provider::webp::WebP, Image};

#[derive(Default)]
pub enum Animation {
    #[default]
    None,
    Gdk(PixbufAnimationIter),
    WebPFile(Box<WebPAnimation<BufReader<File>>>),
    WebPMemory(Box<WebPAnimation<Cursor<Vec<u8>>>>),
}

pub(super) struct AnimationFrame {
    pub(super) delay_ms: u32,
    pub(super) pixbuf: Pixbuf,
}

pub struct WebPAnimation<T> {
    pub(super) decoder: WebPDecoder<T>,
    pub(super) index: u32,
    pub(super) first_run: bool,
    pub(super) frames: Vec<AnimationFrame>,
}

impl Image {
    pub fn is_animation(&self) -> bool {
        !matches!(self.animation, Animation::None)
    }

    pub fn animation_delay_time(&self, ts_previous_cb: SystemTime) -> Option<std::time::Duration> {
        match &self.animation {
            Animation::None => None,
            Animation::Gdk(animation) => animation.delay_time(),
            Animation::WebPFile(animation) => animation.delay_time(ts_previous_cb),
            Animation::WebPMemory(animation) => animation.delay_time(ts_previous_cb),
        }
    }

    pub fn animation_advance(&mut self, current_time: SystemTime) -> bool {
        match &mut self.animation {
            Animation::None => false,
            Animation::Gdk(animation) => {
                if animation.advance(current_time) {
                    self.pixbuf = Some(animation.pixbuf());
                    true
                } else {
                    false
                }
            }
            Animation::WebPFile(animation) => match animation.advance(current_time) {
                Some(pixbuf) => {
                    self.pixbuf = Some(pixbuf);
                    true
                }
                None => false,
            },
            Animation::WebPMemory(animation) => match animation.advance(current_time) {
                Some(pixbuf) => {
                    self.pixbuf = Some(pixbuf);
                    true
                }
                None => false,
            },
        }
    }
}

impl<T: BufRead + Seek> WebPAnimation<T> {
    fn delay_time(&self, ts_previous_cb: SystemTime) -> Option<Duration> {
        if let Some(frame) = self.frames.get(self.index as usize) {
            let interval = Duration::from_millis(frame.delay_ms as u64);
            Some(if let Ok(duration) = ts_previous_cb.elapsed() {
                // dbg!(interval, duration);
                if interval > duration {
                    interval - duration
                } else {
                    Duration::from_millis(1)
                }
            } else {
                interval
            })
        } else {
            None
        }
    }

    fn advance(&mut self, _current_time: SystemTime) -> Option<Pixbuf> {
        self.index += 1;
        if self.index >= self.decoder.num_frames() {
            self.index = 0;
            self.first_run = false;
        }
        if self.first_run {
            if let Ok((pixbuf, delay_ms)) = WebP::read_frame(&mut self.decoder) {
                self.frames.push(AnimationFrame {
                    delay_ms,
                    pixbuf: pixbuf.clone(),
                });
                Some(pixbuf)
            } else {
                None
            }
        } else {
            self.pixbuf_get(self.index as usize)
        }
    }

    pub fn pixbuf_get(&self, index: usize) -> Option<Pixbuf> {
        self.frames.get(index).map(|frame| frame.pixbuf.clone())
    }
}
