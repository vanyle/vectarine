pub mod console;
pub mod game;
pub mod game_resource;
pub mod graphics;
pub mod io;
pub mod loader;
pub mod lua_env;
pub mod math;
pub mod metrics;
pub mod projectinfo;

// Re-export commonly used crates for the editor
pub use anyhow;
pub use mlua;
pub use sdl2;
pub use toml;

use std::{cell::RefCell, mem::ManuallyDrop, rc::Rc, sync::Arc};

use glow::HasContext;
use sdl2::{
    AudioSubsystem, EventPump, Sdl, VideoSubsystem,
    video::{SwapInterval, Window, gl_attr::GLAttr},
};

use crate::game_resource::audio_resource::{AUDIO_CHANNELS, AUDIO_SAMPLE_FREQUENCY};

pub struct RenderingBlock {
    pub video: Rc<RefCell<VideoSubsystem>>,
    pub window: Rc<RefCell<Window>>,
    pub event_pump: EventPump,
    pub sdl: Sdl,
    pub gl: Arc<glow::Context>,
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

pub fn init_audio(sdl: &Sdl) -> Option<AudioSubsystem> {
    let audio = sdl.audio();
    let audio = match audio {
        Ok(audio) => audio,
        Err(audio_err) => {
            println!(
                "Failed to initialize audio subsystem: {:?}. Audio will be disabled.",
                audio_err
            );
            return None;
        }
    };

    let mixer_ctx = sdl2::mixer::init(sdl2::mixer::InitFlag::OGG);
    if let Err(err) = mixer_ctx {
        println!(
            "Failed to initialize audio mixer: {:?}. Audio will be disabled.",
            err
        );
        return None;
    }
    // https://manpages.debian.org/experimental/libsdl3-mixer-doc/Mix_OpenAudioDevice.3.en.html
    // 2048 as a reasonable default. Lower number means lower latency, but you risk dropouts if the number is too low.
    // We use stereo audio (2 channels) and a frequency of 48000 Hz.
    let audio_device = sdl2::mixer::open_audio(
        AUDIO_SAMPLE_FREQUENCY,
        sdl2::mixer::AUDIO_S16,
        AUDIO_CHANNELS,
        1024,
    );
    if let Err(err) = audio_device {
        println!(
            "Failed to open audio device: {:?}. Audio will be disabled.",
            err
        );
        return None;
    }

    // 8 is the default allocated by open_audio
    // sdl2::mixer::allocate_channels(8);

    Some(audio)
}

pub fn deinit_audio(_audio_subsystem: AudioSubsystem) {
    sdl2::mixer::close_audio();
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

    let _gl_context = ManuallyDrop::new(
        window
            .gl_create_context()
            .expect("Failed to create GL context"),
    );

    let gl = make_gl_from_video_system(&video_subsystem);
    let gl: Arc<glow::Context> = Arc::new(gl);

    let _ = video_subsystem.gl_set_swap_interval(SwapInterval::VSync);

    unsafe {
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        //gl.blend_func(glow::SRC_ALPHA_SATURATE, glow::ONE);
        gl.enable(glow::SAMPLE_ALPHA_TO_COVERAGE);
        //gl.enable(glow::POLYGON_SMOOTH);
        gl.enable(glow::MULTISAMPLE);
    }

    RenderingBlock {
        sdl: sdl_context,
        video: Rc::new(RefCell::new(video_subsystem)),
        window: Rc::new(RefCell::new(window)),
        event_pump,
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
    } = init_sdl(|video_subsystem| unsafe {
        glow::Context::from_loader_function(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        })
    });
    let _audio_subsystem = init_audio(&sdl);

    // Initialize IDBFS for persistent storage on Emscripten
    init_fs();

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
