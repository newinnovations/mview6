mod imp;

use gtk4::{
    gio::{self, ApplicationFlags},
    glib, Settings,
};

glib::wrapper! {
    pub struct MviewApplication(ObjectSubclass<imp::MviewApplicationImp>)
        @extends gio::Application, gtk4::Application;
}

impl MviewApplication {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Settings::default()
            .unwrap()
            .set_gtk_application_prefer_dark_theme(true);

        glib::Object::builder()
            .property("application-id", "org.vanderwerff.mview.mview6")
            .property(
                "flags",
                ApplicationFlags::NON_UNIQUE.union(ApplicationFlags::HANDLES_OPEN),
            )
            .build()
    }
}
