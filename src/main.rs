mod application;
mod backends;
mod category;
mod draw;
mod filelist;
mod filelistview;
mod window;

use gdk::Screen;
use gtk::{
    prelude::ApplicationExtManual, prelude::CssProviderExt, CssProvider, StyleContext,
    STYLE_PROVIDER_PRIORITY_USER,
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

    let css_provider = CssProvider::new();
    css_provider
        .load_from_data(include_bytes!("mview6.css"))
        .unwrap();
    StyleContext::add_provider_for_screen(
        &Screen::default().unwrap(),
        &css_provider,
        STYLE_PROVIDER_PRIORITY_USER,
    );

    let app = application::MviewApplication::new();

    app.run();
}
