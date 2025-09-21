pub mod console;
pub mod game;
pub mod game_resource;
pub mod graphics;
pub mod io;
pub mod lua_env;

use std::{cell::RefCell, mem::ManuallyDrop, rc::Rc, sync::Arc};

use glow::HasContext;
use sdl2::{
    EventPump, Sdl, VideoSubsystem,
    video::{Window, gl_attr::GLAttr},
};

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
    gl_attr.set_context_flags().forward_compatible().set(); // for macOS
}

#[cfg(not(target_os = "macos"))]
pub fn set_opengl_attributes(gl_attr: GLAttr) {
    gl_attr.set_context_version(3, 0);
    // gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
}

pub fn init_sdl() -> RenderingBlock {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();

    set_opengl_attributes(gl_attr);

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .opengl()
        .allow_highdpi() // For Retina displays on macOS
        .position_centered()
        .build()
        .unwrap();

    let event_pump = sdl_context.event_pump().unwrap();

    let _gl_context = ManuallyDrop::new(
        window
            .gl_create_context()
            .expect("Failed to create GL context"),
    );

    let gl = unsafe {
        egui_glow::painter::Context::from_loader_function(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        })
    };

    let gl: Arc<glow::Context> = Arc::new(gl);

    unsafe {
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        //gl.blend_func(glow::SRC_ALPHA_SATURATE, glow::ONE);
        //gl.enable(glow::SAMPLE_ALPHA_TO_COVERAGE);
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
