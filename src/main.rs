mod application;
mod backends;
mod category;
mod config;
mod draw;
mod error;
mod filelistview;
mod image;
mod window;

use gdk::Screen;
use gtk::{
    prelude::{ApplicationExtManual, CssProviderExt, IconThemeExt},
    CssProvider, IconTheme, StyleContext, STYLE_PROVIDER_PRIORITY_USER,
};
use std::env;

fn main() {
    gtk::init().expect("Failed to initialize gtk");

    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };
    dbg!(filename);

    gio::resources_register_include!("mview6.gresource").unwrap();

    let css_provider = CssProvider::new();
    css_provider.load_from_resource("/css/mview6.css");
    StyleContext::add_provider_for_screen(
        &Screen::default().unwrap(),
        &css_provider,
        STYLE_PROVIDER_PRIORITY_USER,
    );

    let icon_theme = IconTheme::for_screen(&Screen::default().unwrap()).unwrap();
    icon_theme.add_resource_path("/icons");

    let app = application::MviewApplication::new();

    app.run();
}
