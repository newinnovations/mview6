use cairo::{Context, Format, ImageSurface};
use eog::{Image, ImageExt};

use crate::error::MviewResult;

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
    image.set_zoom_mode(eog::ZoomMode::None);

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
    image.set_zoom_mode(eog::ZoomMode::None);

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
