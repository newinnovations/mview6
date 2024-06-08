use glib::once_cell::unsync::OnceCell;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::window::MViewWindow;

#[derive(Debug, Default)]
pub struct MviewApplicationSub {
    window: OnceCell<MViewWindow>,
}

#[glib::object_subclass]
impl ObjectSubclass for MviewApplicationSub {
    const NAME: &'static str = "MviewApplication";
    type Type = super::MviewApplication;
    type ParentType = gtk::Application;
}

impl ObjectImpl for MviewApplicationSub {}

/// When our application starts, the `startup` signal will be fired.
/// This gives us a chance to perform initialisation tasks that are not directly
/// related to showing a new window. After this, depending on how
/// the application is started, either `activate` or `open` will be called next.
impl ApplicationImpl for MviewApplicationSub {
    /// `gio::Application::activate` is what gets called when the
    /// application is launched by the desktop environment and
    /// aksed to present itself.
    fn activate(&self) {
        let window = self
            .window
            .get()
            .expect("Should always be initiliazed in gio_application_startup");
        window.show_all();
        window.present();
    }

    /// `gio::Application` is bit special. It does not get initialized
    /// when `new` is called and the object created, but rather
    /// once the `startup` signal is emitted and the `gio::Application::startup`
    /// is called.
    ///
    /// Due to this, we create and initialize the `SimpleWindow` widget
    /// here. Widgets can't be created before `startup` has been called.
    fn startup(&self) {
        self.parent_startup();

        let window = MViewWindow::new(&self.obj());
        self.window
            .set(window)
            .expect("Failed to initialize application window");
    }
}

impl GtkApplicationImpl for MviewApplicationSub {}
