use graphics::{text, Transformed};
use graphics_buffer::*;

fn main() {
    // Initalize the buffer
    let mut buffer = RenderBuffer::new(150, 40);
    buffer.clear([0.0, 0.0, 0.0, 1.0]);

    // Load the font and initialize glyphs
    let mut glyphs = buffer_glyphs_from_bytes(include_bytes!("roboto.ttf")).unwrap();

    // Draw text
    text(
        [1.0; 4],
        30,
        "Oh boy!",
        &mut glyphs,
        IDENTITY.trans(10.0, 30.0),
        &mut buffer,
    )
    .unwrap();

    // Save the image
    buffer.save("text.png").unwrap();
}
