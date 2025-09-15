#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::path::Path;
use std::time::Instant;

use runtime::graphics::batchdraw::BatchDraw2d;
use runtime::helpers::game::Game;
use runtime::helpers::lua_env::{self, run_file_and_display_error};
use runtime::{RenderingBlock, init_sdl};

pub fn main() {
    use runtime::helpers::file::read_file;

    let RenderingBlock {
        sdl: _sdl,
        video: _video,
        window,
        event_pump,
        gl,
    } = init_sdl();
    let lua_env = lua_env::LuaEnvironment::new();

    window.borrow_mut().set_resizable(true);

    read_file(
        "assets/scripts/game.lua",
        Box::new(move |content| {
            run_file_and_display_error(&lua_env, &content, Path::new("game.lua"));

            let batch = BatchDraw2d::new(&gl).unwrap();
            let mut game = Game::new(batch, event_pump, lua_env.clone());

            game.load();

            let mut now = Instant::now();

            set_main_loop_wrapper(move || {
                let latest_events = game.event_pump.poll_iter().collect::<Vec<_>>();
                game.load_resource_as_needed(gl.clone());
                game.main_loop(&latest_events, &window, now.elapsed());
                now = Instant::now();

                // These are for debug and are never displayed in the runtime.
                // We still need to clear them to avoid memory leaks.
                {
                    #![cfg(debug_assertions)]
                    for m in game.lua_env.messages.borrow_mut().drain(..) {
                        println!("{m}");
                    }
                }
                game.lua_env.messages.borrow_mut().clear();
                game.lua_env.frame_messages.borrow_mut().clear();

                window.borrow().gl_swap_window();
            });
        }),
    );

    // Prevent exit from destroying the GL context.
    #[cfg(target_os = "emscripten")]
    {
        emscripten_functions::emscripten::exit_with_live_runtime();
    }
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
