use crate::image::colors::Color;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum Category {
    Folder = 0,
    Favorite = 1,
    Image = 2,
    Archive = 3,
    Trash = 4,
    Unsupported = 5,
}

impl Category {
    pub fn determine(filename: &str, is_dir: bool) -> Self {
        if is_dir {
            return Self::Folder;
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

    // https://www.svgrepo.com/svg/347736/file-directory
    // 40% #2ec27e
    //
    // https://www.svgrepo.com/svg/528877/box
    // 70% #62a0ea
    //
    // https://www.svgrepo.com/svg/511024/image-01
    // 70% #f8e45c
    //
    // https://www.svgrepo.com/svg/458675/favorite
    //
    // https://www.svgrepo.com/svg/533010/trash-alt
    // 70% #ffbe6f
    //
    // https://www.svgrepo.com/svg/523073/trash-bin-minimalistic
    // 10% #f66151
    //
    // https://www.svgrepo.com/svg/355272/status-unknown
    // 70% #c0bfbc
    //
    // https://www.svgrepo.com/svg/533035/bookmark

    pub fn icon(&self) -> &str {
        match self {
            Self::Folder => "mv6-folder",
            Self::Favorite => "mv6-favorite",
            Self::Image => "mv6-image",
            Self::Archive => "mv6-box",
            Self::Trash => "mv6-garbage",
            Self::Unsupported => "mv6-unknown",
        }
    }

    pub fn colors(&self) -> (Color, Color, Color) {
        match self {
            Self::Folder => (Color::FolderBack, Color::FolderTitle, Color::FolderMsg),
            Self::Archive => (Color::ArchiveBack, Color::ArchiveTitle, Color::ArchiveMsg),
            Self::Unsupported => (
                Color::UnsupportedBack,
                Color::UnsupportedTitle,
                Color::UnsupportedMsg,
            ),
            _ => (Color::Black, Color::Silver, Color::White),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Category::Folder => "folder",
            Category::Favorite => "favorite",
            Category::Image => "image",
            Category::Archive => "archive",
            Category::Trash => "trash",
            Category::Unsupported => "not supported",
        }
        .into()
    }
}

impl From<u32> for Category {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Folder,
            1 => Self::Favorite,
            2 => Self::Image,
            3 => Self::Archive,
            4 => Self::Trash,
            _ => Self::Unsupported,
        }
    }
}

impl Default for Category {
    fn default() -> Self {
        Self::Unsupported
    }
}
