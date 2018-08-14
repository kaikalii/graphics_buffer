### Description

This library provides a buffer type, `RenderBuffer`, which can be used as a render target for [Piston's graphics library](https://github.com/PistonDevelopers/graphics). This buffer can be loaded from and/or saved to a file on disk. This allows for things like screenshots in games.

[API Documentation](https://docs.rs/graphics_buffer/0.1.0/graphics_buffer/)

### Usage

Add this to your `cargo.toml` :

```toml
graphics_buffer = "0.1.0"
piston2d-graphics = "0.26.0"
```

Here is a simple example that draws three circles and saves the image to a file:

```rust
extern crate graphics;
extern crate graphics_buffer;

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
        identity(),
        &mut buffer,
    );
    // Small blue circle
    ellipse(
        [0.0, 0.0, 1.0, 0.7],
        [0.0, 0.0, 50.0, 50.0],
        identity(),
        &mut buffer,
    );
    // Small green circle
    ellipse(
        [0.0, 1.0, 0.0, 0.7],
        [50.0, 50.0, 50.0, 50.0],
        identity(),
        &mut buffer,
    );

    // Save the buffer
    buffer.save("circles.png").unwrap();
}
```

### Contributing

Feel free to open an issue or PR if you want to contribute. There are definitely places for improvement, especially in the rendering code.
