pub mod draw_instruction;
pub mod file;
pub mod game;
pub mod game_resource;

pub mod lua_env;
///
/// A custom async system supported by emscripten and non-emscripten backends.
pub mod maybelater;

use sdl2::{EventPump, render::Canvas, video::Window};

pub fn init_sdl() -> (Canvas<Window>, EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    // This lines creates an error in the console:  Cannot set timing mode for main loop
    // This is dumb because we need a canvas before setting a main loop!
    let canvas = window.into_canvas().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    (canvas, event_pump)
}
