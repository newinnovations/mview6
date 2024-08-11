mod application;
mod backends;
mod category;
mod config;
mod error;
mod file_view;
mod image;
mod info_view;
mod performance;
mod window;

use gtk4::{
    gdk::Display, prelude::ApplicationExtManual, style_context_add_provider_for_display,
    CssProvider, IconTheme, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use std::env;

fn main() {
    gtk4::init().expect("Failed to initialize gtk");

    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };
    dbg!(filename);

    gio::resources_register_include!("mview6.gresource").unwrap();

    let display = Display::default().expect("Could not connect to a display.");

    let css_provider = CssProvider::new();
    css_provider.load_from_resource("/css/mview6.css");
    style_context_add_provider_for_display(
        &display,
        &css_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_resource_path("/icons");

    let app = application::MviewApplication::new();

    app.run();
}
