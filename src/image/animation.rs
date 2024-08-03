use std::{
    fs::File,
    io::BufReader,
    time::{Duration, SystemTime},
};

use gdk_pixbuf::{Pixbuf, PixbufAnimationIter};
use image_webp::WebPDecoder;

use super::{provider::webp::WebPImage, Image};

#[derive(Default, Debug)]
pub enum Animation {
    #[default]
    None,
    Gdk(PixbufAnimationIter),
    WebP(Box<AnimationFrames>),
}

#[derive(Debug)]
pub(super) struct AnimationFrame {
    pub(super) delay_ms: u32,
    pub(super) pixbuf: Pixbuf,
}

// #[derive(Default, Debug)]
pub struct AnimationFrames {
    pub(super) decoder: WebPDecoder<BufReader<File>>,
    pub(super) index: u32,
    pub(super) first_run: bool,
    pub(super) frames: Vec<AnimationFrame>,
}

impl std::fmt::Debug for AnimationFrames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationFrames")
            .field("index", &self.index)
            .field("frames", &self.frames)
            .finish()
    }
}

impl Image {
    pub fn is_animation(&self) -> bool {
        !matches!(self.animation, Animation::None)
    }

    pub fn animation_delay_time(&self, ts_previous_cb: SystemTime) -> Option<std::time::Duration> {
        match &self.animation {
            Animation::None => None,
            Animation::Gdk(animation) => animation.delay_time(),
            Animation::WebP(animation) => animation.delay_time(ts_previous_cb),
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
            Animation::WebP(animation) => match animation.advance(current_time) {
                Some(pixbuf) => {
                    self.pixbuf = Some(pixbuf);
                    true
                }
                None => false,
            },
        }
    }
}

impl AnimationFrames {
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
            // self.decoder.reset_animation();
        }
        if self.first_run {
            if let Ok((pixbuf, delay_ms)) = WebPImage::read_frame(&mut self.decoder) {
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
