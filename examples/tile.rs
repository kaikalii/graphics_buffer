extern crate find_folder;
extern crate graphics;
extern crate graphics_buffer;
extern crate image;

use graphics::{Image, Transformed};
use graphics_buffer::*;

fn main() {
    let matt = RenderBuffer::from(
        image::open(
            find_folder::Search::ParentsThenKids(3, 3)
                .for_folder("matt.jpg")
                .unwrap(),
        ).unwrap(),
    );
    let mut buffer = RenderBuffer::new(matt.width() * 2, matt.height() * 2);
    buffer.clear([0.0, 0.0, 0.0, 1.0]);
    for (color, (x, y)) in &[
        ([1.0, 0.2, 0.2, 1.0], (0.0, 0.0)),
        ([1.0, 1.0, 0.0, 1.0], (matt.width() as f64, 0.0)),
        ([0.0, 1.0, 0.0, 1.0], (0.0, matt.height() as f64)),
        (
            [0.2, 0.2, 1.0, 1.0],
            (matt.width() as f64, matt.height() as f64),
        ),
    ] {
        Image::new_color(*color).draw(
            &matt,
            &Default::default(),
            identity().trans(*x, *y),
            &mut buffer,
        );
    }
    buffer.save("tiled.png").unwrap();
}
