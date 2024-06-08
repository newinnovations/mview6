mod category;
mod filelist;
mod filelistview;
mod application;
mod window;

use std::env;

use gdk::Screen;
use gtk::prelude::ApplicationExtManual;
use gtk::StyleContext;

use gtk::prelude::CssProviderExt;
use gtk::CssProvider;
use gtk::STYLE_PROVIDER_PRIORITY_USER;

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
