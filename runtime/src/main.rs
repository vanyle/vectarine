#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use runtime::game::Game;
use runtime::io::time::now_ms;
use runtime::{RenderingBlock, init_audio, init_sdl, loader::loader};
use std::panic;

pub fn main() {
    let RenderingBlock {
        sdl,
        video,
        window,
        mut event_pump,
        gl,
    } = init_sdl(|video_subsystem| unsafe {
        glow::Context::from_loader_function(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        })
    });
    let _audio_subsystem = init_audio(&sdl);
    // sdl2::mixer::close_audio(); // no need to clean up, the program will clean on exit.

    loader(move |(project_path, project_info, fs)| {
        Game::from_project(
            &project_path,
            &project_info,
            fs,
            gl,
            &video,
            &window.clone(),
            |result| {
                let Ok(mut game) = result else {
                    panic!("Failed to load the game project at {:?}", project_path);
                };
                let mut now = now_ms();

                set_main_loop_wrapper(move || {
                    let latest_events = event_pump.poll_iter().collect::<Vec<_>>();
                    game.load_resource_as_needed();
                    let now_instant = now_ms();
                    let delta_duration =
                        std::time::Duration::from_micros(((now_instant - now) * 1000.0) as u64);
                    now = now_instant;
                    game.main_loop(&latest_events, &window, delta_duration, false);

                    // These are for debug and are never displayed in the runtime.
                    // We still need to clear them to avoid memory leaks.
                    {
                        #![cfg(debug_assertions)]
                        for m in game.lua_env.messages.borrow_mut().drain(..) {
                            println!("{}", m.msg);
                        }
                    }
                    game.lua_env.messages.borrow_mut().clear();
                    game.lua_env.frame_messages.borrow_mut().clear();

                    window.borrow().gl_swap_window();
                });
            },
        );
    });
    // Prevent exit from destroying the GL context.
    #[cfg(target_os = "emscripten")]
    {
        emscripten_functions::emscripten::exit_with_live_runtime();
    }
}

#[allow(unused_mut)]
fn set_main_loop_wrapper<F>(mut loop_fn: F)
where
    F: FnMut() + 'static,
{
    #[cfg(target_os = "emscripten")]
    {
        emscripten_functions::emscripten::set_main_loop(loop_fn, 0, true);
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        loop {
            loop_fn();
        }
    }
}
