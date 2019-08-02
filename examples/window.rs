use graphics::{ellipse, image, text, Transformed};
use graphics_buffer::*;
use piston_window::{
    clear, Event, Loop, PistonWindow, TextureSettings, UpdateArgs, WindowSettings,
};

fn main() {
    // Load Matt Damon
    let matt = RenderBuffer::decode_from_bytes(include_bytes!("matt.jpg")).unwrap();

    // Load the font and initialize glyphs
    let mut glyphs = buffer_glyphs_from_bytes(include_bytes!("roboto.ttf")).unwrap();

    // Initalize the buffer
    let mut buffer = RenderBuffer::new(matt.width(), matt.height());
    buffer.clear([0.0, 0.0, 0.0, 1.0]);

    // Draw Matt to the buffer
    image(&matt, IDENTITY, &mut buffer);

    // Give Matt red eyes
    const RED: [f32; 4] = [1.0, 0.0, 0.0, 0.7];
    const DIAMETER: f64 = 40.0;
    ellipse(
        RED,
        [115.0, 175.0, DIAMETER, DIAMETER],
        IDENTITY,
        &mut buffer,
    );
    ellipse(
        RED,
        [210.0, 195.0, DIAMETER, DIAMETER],
        IDENTITY,
        &mut buffer,
    );

    // Let people know he is woke
    text(
        [0.0, 1.0, 0.0, 1.0],
        70,
        "# w o k e",
        &mut glyphs,
        IDENTITY.trans(0.0, 70.0),
        &mut buffer,
    )
    .unwrap();

    // Create a window
    let mut window: PistonWindow = WindowSettings::new(
        "piston_window texture example",
        (matt.height(), matt.height()),
    )
    .exit_on_esc(true)
    .build()
    .unwrap();

    // Create a texture from red-eyed Matt
    let matt_texture = buffer
        .to_g2d_texture(
            &mut window.create_texture_context(),
            &TextureSettings::new(),
        )
        .unwrap();

    // Initialize a rotation
    let mut rot = 0.0;

    // Run the event loop
    while let Some(event) = window.next() {
        match event {
            Event::Loop(Loop::Render(..)) => {
                window.draw_2d(&event, |context, graphics, _| {
                    // Clear window with black
                    clear([0.0, 0.0, 0.0, 1.0], graphics);
                    // Draw matt rotated and scaled
                    image(
                        &matt_texture,
                        context
                            .transform
                            .trans(matt.height() as f64 / 2.0, matt.height() as f64 / 2.0)
                            .scale(0.5, 0.5)
                            .rot_rad(rot),
                        graphics,
                    );
                });
            }
            // Rotate on update
            Event::Loop(Loop::Update(UpdateArgs { dt, .. })) => rot += dt,
            _ => (),
        }
    }

    // Save the image
    buffer.save("red_eyes.png").unwrap();
}
