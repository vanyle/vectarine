pub mod helpers;

use sdl2::{EventPump, Sdl, VideoSubsystem, video::Window};

pub fn init_sdl() -> (Sdl, VideoSubsystem, Window, EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    // This lines creates an error in the console:  Cannot set timing mode for main loop
    // This is dumb because we need a canvas before setting a main loop!
    let event_pump = sdl_context.event_pump().unwrap();
    (sdl_context, video_subsystem, window, event_pump)
}
