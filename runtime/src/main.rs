use sdl2::video::Window;
//use notify_debouncer_full::notify::RecursiveMode;
//use notify_debouncer_full::{DebounceEventResult, new_debouncer};
#[cfg(target_os = "emscripten")]
use emscripten_functions::emscripten::set_main_loop;
use std::path::Path;

use crate::lua_env::run_file_and_display_error;

pub mod draw_instruction;
pub mod file;
pub mod game;
pub mod game_resource;

///
/// A custom async system supported by emscripten and non-emscripten backends.
pub mod maybelater;

pub mod lua_env;

pub fn main() {
    maybelater::init_runtime();

    use crate::{file::read_file, game::Game};

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    // This lines creates an error in the console:  Cannot set timing mode for main loop
    // This is dump because we need a canvas before setting a main loop!
    let canvas = window.into_canvas().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    let lua_env = lua_env::LuaEnvironment::new();

    read_file(
        "game.lua",
        Box::new(move |content| {
            run_file_and_display_error(&lua_env, content.as_bytes(), Path::new("game.lua"));

            let mut game = Game::new(canvas, event_pump, lua_env);
            // set_main_loop stops the current function being executed.
            set_main_loop_wrapper(move || game.main_loop());
        }),
    );
}

#[allow(unused_mut)]
fn set_main_loop_wrapper<F>(mut f: F)
where
    F: FnMut() + 'static,
{
    #[cfg(target_os = "emscripten")]
    {
        emscripten_functions::emscripten::set_main_loop(f, 0, true);
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        // TODO: true vsync
        loop {
            f();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}

/*
#[cfg(not(target_os = "emscripten"))]
pub fn main() {
    // Classic entry point.
    // let (debounce_event_sender, debounce_receiver) = channel();

    /*let mut debouncer = new_debouncer(
        Duration::from_millis(100),
        None,
        move |result: DebounceEventResult| match result {
            Ok(events) => events.iter().for_each(|event| {
                let _ = debounce_event_sender.send(event.clone());
            }),
            Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
        },
    )
    .unwrap();*/

    // let lua_for_reload = lua.clone();
    // thread::spawn(move || {
    //     loop {
    //         let event = debounce_receiver.recv();
    //         if let Ok(event) = event {
    //             for path in event.event.paths {
    //                 if path.extension().is_some() && path.extension().unwrap() == "luau" {
    //                     println!("Reloading script: {}", path.to_string_lossy());
    //                     run_file_and_display_error(&lua_for_reload, &path);
    //                 }
    //             }
    //         }
    //     }
    // });

    //debouncer.watch(".", RecursiveMode::NonRecursive).unwrap();
}
*/
