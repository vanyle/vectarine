use std::path::Path;

use runtime::helpers::lua_env::{self, run_file_and_display_error};
use runtime::init_sdl;

pub fn main() {
    use runtime::helpers::{file::read_file, game::Game};

    let (_sdl, _video, window, event_pump) = init_sdl();
    let canvas = window.into_canvas().build().unwrap();
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
