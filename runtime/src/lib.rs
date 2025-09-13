pub mod graphics;
pub mod helpers;

use std::{cell::RefCell, rc::Rc};

use sdl2::{EventPump, Sdl, VideoSubsystem, video::Window};

pub fn init_sdl() -> (
    Sdl,
    Rc<RefCell<VideoSubsystem>>,
    Rc<RefCell<Window>>,
    EventPump,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    // The WebGL backend only supports version 3.0
    // For simplicity, we use 3.0 on all platforms.
    gl_attr.set_context_version(3, 0);

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    // This lines creates an error in the console:  Cannot set timing mode for main loop
    // This is dumb because we need a canvas before setting a main loop!
    let event_pump = sdl_context.event_pump().unwrap();
    (
        sdl_context,
        Rc::new(RefCell::new(video_subsystem)),
        Rc::new(RefCell::new(window)),
        event_pump,
    )
}
