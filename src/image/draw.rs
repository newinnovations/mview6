use cairo::{Context, Format, ImageSurface};
use gdk::pixbuf_get_from_surface;
use gdk_pixbuf::Pixbuf;

use crate::{
    backends::thumbnail::TMessage,
    error::{AppError, MviewError, MviewResult},
    image::{view::ZoomMode, Image},
};

pub fn draw(text: &str) -> MviewResult<Image> {
    let surface = ImageSurface::create(Format::ARgb32, 600, 600)?;
    let context = Context::new(&surface)?;

    // context.set_source_rgb(1.0, 0.2, 0.4);
    context.set_source_rgb(0.0, 0.0, 0.0);
    context.paint()?;

    context.move_to(300.0, 300.0);
    context.set_source_rgb(0.0, 0.0, 0.0);
    for _i in 0..100 {
        context.set_source_rgb(
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        );
        let x = rand::random::<f64>() * 600.0;
        let y = rand::random::<f64>() * 570.0;
        context.line_to(x, y);
        context.stroke()?;
        context.move_to(x, y);
    }

    context.set_font_size(20.0);
    let margin = 10.0;

    // context.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    // context.select_font_face("Ubuntu", cairo::FontSlant::Normal, cairo::FontWeight::Normal); //Bold);

    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    ); //Bold);
    let extends = context.text_extents(text)?;

    // dbg!(extends);

    let text_x = 300.0 - extends.width() / 2.0;
    let text_y = 300.0;

    let box_x = text_x - margin;
    let box_y_top = text_y + extends.y_bearing() - margin;
    let box_y_bottom = box_y_top + extends.height() + 2.0 * margin;

    context.set_source_rgb(0.2, 0.0, 0.2);
    context.rectangle(box_x, box_y_top, 600.0 - box_x, box_y_bottom - box_y_top);
    context.fill()?;

    context.set_source_rgb(0.9, 0.8, 0.2);
    context.move_to(text_x, text_y);
    context.show_text(text)?;

    context.move_to(box_x, box_y_top);
    context.line_to(box_x, box_y_bottom);
    context.move_to(box_x, box_y_top);
    context.line_to(600.0, box_y_top);

    context.move_to(box_x, box_y_bottom);
    context.line_to(600.0, box_y_bottom);

    context.stroke()?;

    logo(&context, 595, 598, 25.0)?;

    // context.select_font_face(
    //     "Liberation Sans",
    //     cairo::FontSlant::Normal,
    //     cairo::FontWeight::Bold,
    // );
    // let extends = context.text_extents("MView6")?;
    // context.move_to(595.0 - extends.width(), 598.0);
    // context.set_source_rgb(1.0, 0.0, 0.0);
    // context.show_text("M")?;
    // context.set_source_rgb(1.0, 1.0, 1.0);
    // context.show_text("View6")?;

    // context.stroke()?;

    let image = Image::new_image_surface(&surface);
    image.set_zoom_mode(ZoomMode::None);

    Ok(image)
}

pub fn thumbnail_sheet(width: i32, height: i32, offset_x: i32, text: &str) -> MviewResult<Image> {
    let surface: ImageSurface = ImageSurface::create(Format::ARgb32, width, height)?;
    let context = Context::new(&surface)?;

    // context.set_source_rgb(1.0, 0.2, 0.4);
    context.set_source_rgb(0.0, 0.0, 0.0);
    context.paint()?;

    logo(&context, width - offset_x, height - 15, 30.0)?;

    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );
    context.set_font_size(20.0);
    let extends = context.text_extents(text)?;
    context.move_to(width as f64 / 2.0 - extends.width(), height as f64 - 18.0);
    context.set_source_rgb(1.0, 1.0, 1.0);
    context.show_text(text)?;

    let image = Image::new_image_surface(&surface);
    image.set_zoom_mode(ZoomMode::None);

    Ok(image)
}

fn logo(context: &Context, x: i32, y: i32, size: f64) -> MviewResult<()> {
    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Bold,
    );
    context.set_font_size(size);
    let extends = context.text_extents("MView6")?;
    context.move_to(x as f64 - extends.width(), y as f64);
    context.set_source_rgb(1.0, 0.0, 0.0);
    context.show_text("M")?;
    context.set_source_rgb(1.0, 1.0, 1.0);
    context.show_text("View6")?;
    context.stroke()?;
    Ok(())
}

pub fn text_thumb(message: TMessage) -> MviewResult<Pixbuf> {
    let surface: ImageSurface = ImageSurface::create(Format::ARgb32, 175, 175)?;
    let context = Context::new(&surface)?;

    // context.set_source_rgb(0.1, 0.1, 0.2);
    context.set_source_rgb(0.2, 0.1, 0.1);
    context.paint()?;

    // logo(&context, width - offset_x, height - 15, 30.0)?;

    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Bold,
    );
    context.set_font_size(20.0);
    let extends = context.text_extents(message.title())?;
    context.move_to((175.0 - extends.width()) / 2.0, 60.0);
    context.set_source_rgb(1.0, 1.0, 1.0);
    context.show_text(message.title())?;

    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );
    context.set_font_size(14.0);
    context.set_source_rgb(1.0, 1.0, 1.0);

    let target_width = 160.0;

    let extends = context.text_extents(message.message())?;

    if extends.width() > target_width {
        let msg = message.message().chars().collect::<Vec<char>>();

        let mid = msg.len() / 2;

        let mut chars_lost = false;

        let mut m = mid;
        let mut first;
        let mut first_extends;
        loop {
            let a = &msg[..m];
            first = a.iter().collect::<String>();
            first_extends = context.text_extents(&first)?;
            if first_extends.width() <= target_width || m == 0 {
                break;
            }
            chars_lost = true;
            m -= 1;
        }

        let mut m = mid;
        let mut second;
        let mut second_extends;
        loop {
            let a = &msg[m..];
            second = a.iter().collect::<String>();
            second_extends = context.text_extents(&second)?;
            m += 1;
            if second_extends.width() <= target_width || m == msg.len() {
                break;
            }
            chars_lost = true;
        }

        if chars_lost {
            context.move_to(80.0, 121.0);
            context.show_text("...")?;
            context.move_to((175.0 - first_extends.width()) / 2.0, 110.0);
            context.show_text(&first)?;
            context.move_to((175.0 - second_extends.width()) / 2.0, 140.0);
            context.show_text(&second)?;
        } else {
            context.move_to((175.0 - first_extends.width()) / 2.0, 110.0);
            context.show_text(&first)?;
            context.move_to((175.0 - second_extends.width()) / 2.0, 135.0);
            context.show_text(&second)?;
        }
    } else {
        context.move_to((175.0 - extends.width()) / 2.0, 110.0);
        context.show_text(message.message())?;
    }

    match pixbuf_get_from_surface(&surface, 0, 0, 175, 175) {
        Some(pixbuf) => Ok(pixbuf),
        None => Err(MviewError::App(AppError::new(
            "Failed to get pixbuf from surface",
        ))),
    }
}
