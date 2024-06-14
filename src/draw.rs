use cairo::{Context, Error, Format, ImageSurface};
use eog::Image;

pub fn draw() -> Result<Image, Error> {
    let surface = ImageSurface::create(Format::ARgb32, 600, 600)?;
    let context = Context::new(&surface)?;

    // context.set_source_rgb(1.0, 0.2, 0.4);
    context.set_source_rgb(0.0, 0.0, 0.0);
    context.paint().expect("paint");

    context.move_to(300.0, 300.0);
    context.set_source_rgb(0.0, 0.0, 0.0);
    for _i in 0..100 {
        context.set_source_rgb(
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        );
        let x = rand::random::<f64>() * 600.0;
        let y = rand::random::<f64>() * 600.0;
        context.line_to(x, y);
        context.stroke()?;
        context.move_to(x, y);
    }

    context.set_source_rgb(1.0, 1.0, 0.0);

    context.set_font_size(25.0);

    // context.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    context.select_font_face("Ubuntu", cairo::FontSlant::Normal, cairo::FontWeight::Bold);

    let text = "MView6 - Advanced Image Viewer";

    let extends = context.text_extents(text)?;

    dbg!(extends);

    let x = 300.0 - extends.width() / 2.0;
    let y = 300.0;

    context.move_to(x, y);
    context.show_text(text)?;
    context.move_to(x, 0.0);
    context.line_to(x, 600.0);
    context.move_to(0.0, y);
    context.line_to(600.0, y);

    let y = 300.0 + extends.y_bearing();
    context.move_to(0.0, y);
    context.line_to(600.0, y);

    context.stroke()?;

    Ok(Image::new_image_surface(&surface))
}
