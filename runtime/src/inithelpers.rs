use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::sync::Arc;

use sdl2::video::Window;
use sdl2::video::gl_attr::GLAttr;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use vectarine_plugin_sdk::{glow, sdl2};

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
    // gl_attr.set_context_profile(vectarine_plugin_sdk::sdl2::video::GLProfile::Core);
}

/// A datastructure that holds the primitives needed to interact with the environment. (windows, graphics, io, sound, etc.)
pub struct RenderingBlock {
    pub video: Rc<VideoSubsystem>,
    pub window: Rc<RefCell<Window>>,
    pub event_pump: EventPump,
    pub sdl: Sdl,
    pub gl: Arc<glow::Context>,
    pub gl_context: ManuallyDrop<sdl2::video::GLContext>,
}
