use std::path::Path;
use std::sync::Arc;

use runtime::graphics::batchdraw::BatchDraw2d;
use runtime::helpers::game::Game;
use runtime::helpers::lua_env::{self, run_file_and_display_error};
use runtime::init_sdl;

pub fn main() {
    use runtime::helpers::file::read_file;

    let (_sdl, video, window, event_pump) = init_sdl();
    let lua_env = lua_env::LuaEnvironment::new();

    let _gl_context = window
        .borrow()
        .gl_create_context()
        .expect("Failed to create GL context");

    let gl = unsafe {
        egui_glow::painter::Context::from_loader_function(|name| {
            video.borrow().gl_get_proc_address(name) as *const _
        })
    };

    let gl: Arc<glow::Context> = Arc::new(gl);

    read_file(
        "assets/scripts/game.lua",
        Box::new(move |content| {
            run_file_and_display_error(&lua_env, content.as_bytes(), Path::new("game.lua"));

            // let drawing_target = DrawingTarget::new(&gl);
            let batch = BatchDraw2d::new(&gl).unwrap();

            let mut game = Game::new(batch, event_pump, lua_env);

            set_main_loop_wrapper(move || {
                let latest_events = game.event_pump.poll_iter().collect::<Vec<_>>();
                game.main_loop(&latest_events);
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
