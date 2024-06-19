use gtk::ListStore;

pub mod archive_rar;
pub mod archive_zip;
pub mod filesystem;

#[derive(Debug)]
#[repr(u32)]
pub enum Columns {
    Cat = 0,
    Icon,
    Name,
    Size,
    Modified,
}

pub trait Backend {
    fn class_name(&self) -> &str;
    fn create_store(&self) -> Option<ListStore>;
    // load image
    // enter
    // leave
    // favorite
}

impl std::fmt::Debug for dyn Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Backend({})", self.class_name())
    }
}
