extern crate graphics;
extern crate graphics_buffer;

use graphics::{Image, Transformed};
use graphics_buffer::*;

fn main() {
    // Load Matt Damon
    let matt = RenderBuffer::decode_from_bytes(include_bytes!("matt.jpg")).unwrap();

    // Initalize the buffer
    let mut buffer = RenderBuffer::new(matt.width() * 2, matt.height() * 2);
    buffer.clear([0.0, 0.0, 0.0, 1.0]);

    // Tile the image with different colors
    for (color, (x, y)) in &[
        ([1.0, 0.2, 0.2, 1.0], (0.0, 0.0)), // red, top left
        ([1.0, 1.0, 0.0, 1.0], (matt.width() as f64, 0.0)), // yellow, top right
        ([0.0, 1.0, 0.0, 1.0], (0.0, matt.height() as f64)), // green, bottom left
        (
            [0.2, 0.2, 1.0, 1.0],                        // blue
            (matt.width() as f64, matt.height() as f64), // bottom right
        ),
    ] {
        Image::new_color(*color).draw(
            &matt,
            &Default::default(),
            identity().trans(*x, *y),
            &mut buffer,
        );
    }

    // Save the image
    buffer.save("tiled.png").unwrap();
}
