pub mod console;
pub mod game;
pub mod game_resource;
pub mod graphics;
pub mod io;
pub mod loader;
pub mod lua_env;
pub mod math;
pub mod metrics;
pub mod native_plugin;
pub mod projectinfo;
pub mod sound;

// Re-export commonly used crates for the editor
pub use anyhow;
pub use egui;
pub use egui_glow;
pub use mlua;
pub use sdl2;
pub use toml;

use std::{cell::RefCell, mem::ManuallyDrop, rc::Rc, sync::Arc};

use sdl2::{
    EventPump, Sdl, VideoSubsystem,
    video::{SwapInterval, Window, gl_attr::GLAttr},
};

use crate::{
    game_resource::audio_resource::{AUDIO_CHANNELS, AUDIO_SAMPLE_FREQUENCY},
    native_plugin::{NativePlugin, plugininterface::PluginInterface},
    sound::init_sound_system,
};

pub struct RenderingBlock {
    pub video: Rc<RefCell<VideoSubsystem>>,
    pub window: Rc<RefCell<Window>>,
    pub event_pump: EventPump,
    pub sdl: Sdl,
    pub gl: Arc<glow::Context>,
    pub gl_context: ManuallyDrop<sdl2::video::GLContext>,
}

pub fn get_shader_version() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "#version 330 core"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "#version 300 es"
    }
}

#[cfg(target_os = "macos")]
pub fn set_opengl_attributes(gl_attr: GLAttr) {
    // MacOS does not support OpenGL ES.
    gl_attr.set_context_version(3, 0);
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(4);
    gl_attr.set_stencil_size(8); // Request 8-bit stencil buffer
    gl_attr.set_context_flags().forward_compatible().set(); // for macOS
}

#[cfg(not(target_os = "macos"))]
pub fn set_opengl_attributes(gl_attr: GLAttr) {
    gl_attr.set_context_version(3, 0);
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(4);
    gl_attr.set_stencil_size(8); // Request 8-bit stencil buffer
    // gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
}

pub fn init_sdl<F>(make_gl_from_video_system: F) -> RenderingBlock
where
    F: FnOnce(&VideoSubsystem) -> glow::Context,
{
    let sdl_context = sdl2::init().expect("Failed to initialize SDL");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to initialize video subsystem");
    let gl_attr = video_subsystem.gl_attr();

    set_opengl_attributes(gl_attr);

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .opengl()
        .allow_highdpi() // For Retina displays on macOS
        .position_centered()
        .build()
        .expect("Failed to create window");

    let event_pump = sdl_context
        .event_pump()
        .expect("Failed to create event pump");

    let gl_context = ManuallyDrop::new(
        window
            .gl_create_context()
            .expect("Failed to create GL context"),
    );

    // window.gl_make_current(&_gl_context);

    let gl = make_gl_from_video_system(&video_subsystem);
    let gl: Arc<glow::Context> = Arc::new(gl);

    let _ = video_subsystem.gl_set_swap_interval(SwapInterval::VSync);

    RenderingBlock {
        sdl: sdl_context,
        video: Rc::new(RefCell::new(video_subsystem)),
        window: Rc::new(RefCell::new(window)),
        event_pump,
        gl_context,
        gl,
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
        emscripten_functions::emscripten::set_main_loop(loop_fn, 0, true);
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
        video,
        window,
        mut event_pump,
        gl,
        ..
    } = init_sdl(|video_subsystem| unsafe {
        glow::Context::from_loader_function(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        })
    });
    init_sound_system(&sdl);

    // Initialize IDBFS for persistent storage on Emscripten
    init_fs();

    let native_plugin = NativePlugin::load("libeditor_plugin_template.dylib")
        .expect("The plugin could not be loaded");

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

                let plugin_interface = PluginInterface::new(&game.lua_env.lua, 3);
                native_plugin.call_init_hook(plugin_interface);

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
                        console::consume_logs(|log| {
                            println!("{}", log);
                        });
                        console::consume_frame_logs(|log| {
                            println!("{}", log);
                        });
                    }
                    console::clear_all_logs();

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
