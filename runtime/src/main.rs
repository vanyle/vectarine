#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::path::Path;
use std::time::Instant;

use runtime::game::Game;
use runtime::game_resource::script_resource::ScriptResource;
use runtime::graphics::batchdraw::BatchDraw2d;
use runtime::lua_env::{self};
use runtime::{RenderingBlock, init_sdl};

pub fn main() {
    let RenderingBlock {
        sdl: _sdl,
        video,
        window,
        event_pump,
        gl,
    } = init_sdl();
    let lua_env = lua_env::LuaEnvironment::new();

    let path = Path::new("scripts/game.lua");
    lua_env
        .resources
        .load_resource::<ScriptResource>(path, lua_env.lua.clone(), gl.clone());

    let batch = BatchDraw2d::new(&gl).unwrap();
    let mut game = Game::new(&gl, batch, event_pump, lua_env.clone());

    game.load(&video, &window);

    let mut now = Instant::now();

    set_main_loop_wrapper(move || {
        let latest_events = game.event_pump.poll_iter().collect::<Vec<_>>();
        game.load_resource_as_needed(gl.clone());
        game.main_loop(&latest_events, &window, now.elapsed(), false);
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
