extern crate graphics;
extern crate graphics_buffer;
extern crate image;

use graphics::ellipse;
use graphics_buffer::*;

fn main() {
    let mut buffer = RenderBuffer::new(100, 100);
    buffer.clear([0.0, 0.0, 0.0, 1.0]);
    ellipse(
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 50.0, 50.0],
        identity(),
        &mut buffer,
    );
    buffer.save("circle.png").unwrap();
}
