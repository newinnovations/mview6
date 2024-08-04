#!/usr/bin/env python3

# https://coolors.co

COLORS = [
    ("white", "#ffffff"),
    ("black", "#000000"),
    ("red", "#ff0000"),
    ("black_bean", "#331a1a"),
    ("gray", "#808080"),
    ("silver", "#cccccc"),
    ("russian_violet", "#330033"),
    ("citrine", "#e6cc33"),
    ("folder_back", "#1e3319"),
    ("folder_title", "#94bc8a"),
    ("folder_msg", "#c0cebf"),
    ("archive_back", "#23213c"),
    ("archive_title", "#89a6d2"),
    ("archive_msg", "#c3ccd9"),
    ("unsupported_back", "#292500"),
    ("unsupported_title", "#c4b850"),
    ("unsupported_msg", "#d0cc9f"),
    ("error_back", "#372020"),
    ("error_title", "#b38888"),
    ("error_msg", "#dbb5b5"),
]


CHUNK1 = """use cairo::Context;

// generated file - do not edit

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
"""

CHUNK2 = """}

pub trait CairoColorExt {
    fn color(&self, color: Color);
}

impl CairoColorExt for Context {
    fn color(&self, color: Color) {
        match color {
"""

CHUNK3 = """        }
    }
}
"""


def camel_case(word):
    return "".join(x.capitalize() or "_" for x in word.split("_"))


def main(f):
    f.write(CHUNK1)
    for name, color in COLORS:
        name = camel_case(name)
        f.write(
            f"""    {name}, // {color.lower()}
"""
        )
    f.write(CHUNK2)
    for name, color in COLORS:
        name = camel_case(name)
        hex = [b / 255.0 for b in bytes.fromhex(color.replace("#", ""))]
        if len(hex) == 4:
            r, g, b, a = hex
            f.write(
                f"""            Color::{name} => {{
                self.set_source_rgba({r:.2f}, {g:.2f}, {b:.2f}, {a:.2f}); // {color.lower()}
            }}
"""
            )
        elif len(hex) == 3:
            r, g, b = hex
            f.write(
                f"""            Color::{name} => {{
                self.set_source_rgb({r:.2f}, {g:.2f}, {b:.2f}); // {color.lower()}
            }}
"""
            )
    f.write(CHUNK3)


if __name__ == "__main__":
    with open("../src/image/colors.rs", "w") as f:
        main(f)
