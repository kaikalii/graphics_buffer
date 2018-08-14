extern crate find_folder;
extern crate graphics;
extern crate graphics_buffer;

use std::{fs::File, io::Read};

use graphics::{text, Transformed};
use graphics_buffer::*;

fn main() {
    // Initalize the buffer
    let mut buffer = RenderBuffer::new(100, 40);
    buffer.clear([0.0, 0.0, 0.0, 1.0]);

    // Load the font and initialize glyphs
    let mut font_data = Vec::new();
    let font_path = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("roboto.ttf")
        .unwrap();
    File::open(font_path)
        .unwrap()
        .read_to_end(&mut font_data)
        .unwrap();
    let mut glyphs = BufferGlyphs::from_bytes(&font_data).unwrap();

    // Draw text
    text(
        [1.0, 1.0, 1.0, 1.0],
        30,
        "Oh boy!",
        &mut glyphs,
        identity().trans(10.0, 30.0),
        &mut buffer,
    ).unwrap();

    // Save the image
    buffer.save("text.png").unwrap();
}
