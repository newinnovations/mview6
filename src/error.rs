use std::fmt;

use unrar::error::UnrarError;
use zip::result::ZipError;

pub struct AppError {
    msg: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MView6: {}", self.msg)
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ file: {}, line: {}, msg: {} }}",
            file!(),
            line!(),
            self.msg
        )
    }
}

impl AppError {
    pub fn new(msg: &str) -> Self {
        AppError {
            msg: msg.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum MviewError {
    App(AppError),

    Image(image::ImageError),

    Cairo(cairo::Error),

    Io(std::io::Error),

    Zip(ZipError),

    Rar(UnrarError),

    Glib(glib::Error),
}

impl From<std::io::Error> for MviewError {
    fn from(err: std::io::Error) -> MviewError {
        MviewError::Io(err)
    }
}

impl From<ZipError> for MviewError {
    fn from(err: ZipError) -> MviewError {
        MviewError::Zip(err)
    }
}

impl From<UnrarError> for MviewError {
    fn from(err: UnrarError) -> MviewError {
        MviewError::Rar(err)
    }
}

impl From<cairo::Error> for MviewError {
    fn from(err: cairo::Error) -> MviewError {
        MviewError::Cairo(err)
    }
}

impl From<image::ImageError> for MviewError {
    fn from(err: image::ImageError) -> MviewError {
        MviewError::Image(err)
    }
}

impl From<glib::Error> for MviewError {
    fn from(err: glib::Error) -> MviewError {
        MviewError::Glib(err)
    }
}

impl fmt::Display for MviewError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            MviewError::App(err) => err.fmt(fmt),
            MviewError::Io(err) => err.fmt(fmt),
            MviewError::Zip(err) => err.fmt(fmt),
            MviewError::Rar(err) => err.fmt(fmt),
            MviewError::Cairo(err) => err.fmt(fmt),
            MviewError::Image(err) => err.fmt(fmt),
            MviewError::Glib(err) => err.fmt(fmt),
        }
    }
}

pub type MviewResult<T> = Result<T, MviewError>;
