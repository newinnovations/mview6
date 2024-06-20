use gio::File;
use glib::once_cell::unsync::OnceCell;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::window::MViewWindow;

#[derive(Debug, Default)]
pub struct MviewApplicationImp {
    window: OnceCell<MViewWindow>,
}

#[glib::object_subclass]
impl ObjectSubclass for MviewApplicationImp {
    const NAME: &'static str = "MviewApplication";
    type Type = super::MviewApplication;
    type ParentType = gtk::Application;
}

impl ObjectImpl for MviewApplicationImp {}

/// When our application starts, the `startup` signal will be fired.
/// This gives us a chance to perform initialisation tasks that are not directly
/// related to showing a new window. After this, depending on how
/// the application is started, either `activate` or `open` will be called next.
impl ApplicationImpl for MviewApplicationImp {
    /// `gio::Application::activate` is what gets called when the
    /// application is launched by the desktop environment and
    /// aksed to present itself.
    fn activate(&self) {
        let window = self.window.get().expect("failed to get window");
        println!("window:activate");
        window.show_all();
        window.present();
    }

    fn startup(&self) {
        self.parent_startup();
        let window = MViewWindow::new(&self.obj());
        self.window
            .set(window)
            .expect("Failed to initialize application window");
    }

    fn open(&self, files: &[File], hint: &str) {
        println!("OPEN");
        dbg!(files, hint);
        if !files.is_empty() {
            let file = &files[0];
            let window = self.window.get().expect("failed to get window");
            // window.load(file);
            window.navigate_to(file);
        }
    }
}

impl GtkApplicationImpl for MviewApplicationImp {}
