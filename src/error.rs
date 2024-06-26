use std::fmt;

#[derive(Debug)]
pub enum MviewError {
    Image(image::ImageError),

    Cairo(cairo::Error),

    Io(std::io::Error),

    Glib(glib::Error),
}

impl From<std::io::Error> for MviewError {
    fn from(err: std::io::Error) -> MviewError {
        MviewError::Io(err)
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
            MviewError::Io(err) => err.fmt(fmt),
            MviewError::Cairo(err) => err.fmt(fmt),
            MviewError::Image(err) => err.fmt(fmt),
            MviewError::Glib(err) => err.fmt(fmt),
        }
    }
}

pub type MviewResult<T> = Result<T, MviewError>;
