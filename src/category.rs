#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum Category {
    Direcory = 0,
    Favorite = 1,
    Image = 2,
    Archive = 3,
    Trash = 4,
    Unsupported = 5,
}

impl Category {
    pub fn determine(filename: &str, is_dir: bool) -> Self {
        if is_dir {
            return Self::Direcory;
        }

        let filename_lower = filename.to_lowercase();

        let archive = filename_lower.ends_with(".zip")
            | filename_lower.ends_with(".rar")
            | filename_lower.ends_with(".tar")
            | filename_lower.contains(".tar.")
            | filename_lower.ends_with(".tgz");

        if archive {
            return Self::Archive;
        }

        let supported = filename_lower.ends_with(".jpg")
            | filename_lower.ends_with(".jpeg")
            | filename_lower.ends_with(".gif")
            | filename_lower.ends_with(".svg")
            | filename_lower.ends_with(".webp")
            | filename_lower.ends_with("-1")
            | filename_lower.ends_with(".png");

        if supported {
            if filename_lower.contains(".hi.") {
                return Self::Favorite;
            }
            if filename_lower.contains(".lo.") {
                return Self::Trash;
            }
            Self::Image
        } else {
            Self::Unsupported
        }
    }

    pub fn id(&self) -> u32 {
        *self as u32
    }

    pub fn from(id: u32) -> Self {
        match id {
            0 => Self::Direcory,
            1 => Self::Favorite,
            2 => Self::Image,
            3 => Self::Archive,
            4 => Self::Trash,
            _ => Self::Unsupported,
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Direcory => "folder",
            Self::Favorite => "emblem-favorite", // "starred"
            Self::Image => "image-x-generic",
            Self::Archive => "package-x-generic",
            Self::Trash => "user-trash",
            Self::Unsupported => "text-x-generic",
        }
    }
}
