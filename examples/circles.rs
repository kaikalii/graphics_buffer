use graphics::ellipse;
use graphics_buffer::*;

fn main() {
    // Create a new RenderBuffer
    let mut buffer = RenderBuffer::new(100, 100);
    buffer.clear([0.0, 0.0, 0.0, 0.0]);

    // Big red circle
    ellipse(
        [1.0, 0.0, 0.0, 0.7],
        [0.0, 0.0, 100.0, 100.0],
        IDENTITY,
        &mut buffer,
    );
    // Small blue circle
    ellipse(
        [0.0, 0.0, 1.0, 0.7],
        [0.0, 0.0, 50.0, 50.0],
        IDENTITY,
        &mut buffer,
    );
    // Small green circle
    ellipse(
        [0.0, 1.0, 0.0, 0.7],
        [50.0, 50.0, 50.0, 50.0],
        IDENTITY,
        &mut buffer,
    );

    // Save the buffer
    buffer.save("circles.png").unwrap();
}
