### Description

This library provides a buffer type, `RenderBuffer`, which can be used as a render target for [Piston's graphics library](https://github.com/PistonDevelopers/graphics). This buffer can be loaded from and/or saved to a file on disk. This allows for things like screenshots in games.

There is also an optional feature for `RenderBuffer` that allows it to be converted into a `G2dTexture` so that it can be rendered with [`piston_window`](https://github.com/PistonDevelopers/piston_window). To enable this, add `features = ["piston_window_texture"]` to the `graphics_buffer` dependency in your `cargo.toml`.

[API Documentation](https://docs.rs/graphics_buffer/)

### Usage

Add this to your `cargo.toml` :

```toml
graphics_buffer = "0.4.2"
piston2d-graphics = "0.26.0"
```

or, if you want to be able to draw the texture to a window using [`piston_window`](https://github.com/PistonDevelopers/piston_window) :

```toml
graphics_buffer = { version = "0.4.2", features = ["piston_window_texture"] }
piston2d-graphics = "0.26.0"
piston_window = "0.82.0"
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
