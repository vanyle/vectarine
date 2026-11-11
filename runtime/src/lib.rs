pub mod async_handler;
pub mod console;
pub mod drawing_surface;
pub mod game;
pub mod game_resource;
pub mod gamepad;
pub mod graphics;
pub mod inithelpers;
pub mod io;
pub mod loader;
pub mod lua_env;
pub mod math;
pub mod metrics;
pub mod native_plugin;
pub mod projectinfo;
pub mod sound;

use crate::drawing_surface::DrawingSurface;
use crate::drawing_surface::SurfaceMargins;
use crate::drawing_surface::sdl_drawing_surface::SdlDrawingSurface;
use crate::drawing_surface::sdl_drawing_surface::drawable_screen_size;
use crate::gamepad::{DummyGamepadSystem, GamepadSystem};
use crate::graphics::gfx::Gfx;
use crate::graphics::gpu::Gpu;
use crate::graphics::surface::WindowSurface;
use crate::inithelpers::RenderingBlock;

// Re-export commonly used crates for the editor
pub use image;
pub use vectarine_plugin_sdk::anyhow;
pub use vectarine_plugin_sdk::egui;
pub use vectarine_plugin_sdk::egui_wgpu;
pub use vectarine_plugin_sdk::lazy_static;
pub use vectarine_plugin_sdk::mlua;
pub use vectarine_plugin_sdk::rapier2d;
pub use vectarine_plugin_sdk::sdl2;
pub use vectarine_plugin_sdk::serde;
pub use vectarine_plugin_sdk::toml;
pub use vectarine_plugin_sdk::wgpu;

use std::{cell::RefCell, rc::Rc};

use sdl2::video::Window;

use crate::{
    game_resource::audio_resource::{AUDIO_CHANNELS, AUDIO_SAMPLE_FREQUENCY},
    sound::init_sound_system,
};

pub fn init_sdl() -> RenderingBlock {
    let sdl_context = sdl2::init().expect("Failed to initialize SDL");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to initialize video subsystem");

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .allow_highdpi() // For Retina displays on macOS
        .position_centered()
        .metal_view()
        .build()
        .expect("Failed to create window");

    let event_pump = sdl_context
        .event_pump()
        .expect("Failed to create event pump");

    // Safety: the window is kept in the RenderingBlock with its surface.
    let (gpu, surface) =
        unsafe { Gpu::for_window(&window) }.expect("Failed to initialize the graphics");
    let (width, height) = drawable_screen_size(&window);
    let surface = WindowSurface::new(&gpu, surface, width, height);
    let gfx = Gfx::new(gpu);

    let window = Rc::new(RefCell::new(SdlDrawingSurface::new(
        window,
        &video_subsystem,
        None,
    )));

    window
        .borrow_mut()
        .set_drawable_size_in_logical_px(800, 600);

    let gamepad_system = sdl_context.game_controller();
    let gamepad_system: Rc<dyn GamepadSystem> = match gamepad_system {
        Ok(system) => Rc::new(system),
        Err(_) => Rc::new(DummyGamepadSystem),
    };

    RenderingBlock {
        sdl: sdl_context,
        window,
        event_pump,
        gfx,
        surface,
        video_subsystem,
        gamepad_system,
    }
}

/// Wrapper for setting up the main loop, handling differences between Emscripten and native
#[allow(unused_mut)]
pub fn set_main_loop_wrapper<F>(mut loop_fn: F)
where
    F: FnMut() + 'static,
{
    #[cfg(target_os = "emscripten")]
    {
        // Setting simulate_infinite_loop to true throws an exception which we cannot catch (this is bad).
        emscripten_functions::emscripten::set_main_loop(loop_fn, 0, false);
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        loop {
            loop_fn();
        }
    }
}

/// Main library entry point for the runtime
/// This can be called from main.rs or other binaries like the editor
pub fn lib_main() {
    use crate::game::Game;
    use crate::io::fs::init_fs;
    use crate::io::time::now_ms;
    use crate::loader::loader;

    let RenderingBlock {
        sdl,
        window,
        mut event_pump,
        gfx,
        mut surface,
        gamepad_system,
        ..
    } = init_sdl();
    init_sound_system(&sdl);

    // Initialize IDBFS for persistent storage on Emscripten
    init_fs();

    let drawing_surface: Rc<RefCell<dyn DrawingSurface>> = window;

    loader(move |(project_path, project_info, fs)| {
        Game::from_project(
            &project_path,
            &project_info,
            fs,
            gfx.clone(),
            &drawing_surface.clone(),
            gamepad_system,
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

                    let (width, height) =
                        drawing_surface.borrow().get_drawable_size_in_hardware_px();
                    let is_visible = surface.begin_frame(&gfx, width, height);
                    game.main_loop(
                        latest_events.iter(),
                        &drawing_surface,
                        delta_duration,
                        false,
                        SurfaceMargins::default(),
                    );

                    // These are for debug and are never displayed in the runtime.
                    // We still need to clear them to avoid memory leaks.
                    #[allow(unused_variables)]
                    {
                        console::consume_logs(|log| {
                            #[cfg(debug_assertions)]
                            println!("{}", log);
                        });
                        console::consume_frame_logs(|log| {
                            #[cfg(debug_assertions)]
                            println!("{}", log);
                        });
                    }
                    console::clear_all_logs();

                    surface.end_frame(&gfx);

                    // Nothing waits for the screen when the window is hidden
                    #[cfg(not(target_os = "emscripten"))]
                    if !is_visible {
                        std::thread::sleep(std::time::Duration::from_millis(16));
                    }
                    #[cfg(target_os = "emscripten")]
                    let _ = is_visible;
                });
            },
        );
    });

    // Prevent exit from destroying the graphics context.
    #[cfg(target_os = "emscripten")]
    {
        emscripten_functions::emscripten::exit_with_live_runtime();
    }
}
