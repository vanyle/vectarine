pub mod batchdraw;
pub mod customshader;
#[cfg(target_os = "emscripten")]
pub mod emscripten_egl;
pub mod framebuffer;
pub mod gfx;
pub mod gpu;
pub mod pipeline;
pub mod stencil;
pub mod surface;
pub mod texture;

pub mod shadersources;
pub mod shape;

pub mod affinetransform;

pub const VECTARINE_LOGO_PNG: &[u8] = include_bytes!("../../assets/logo.png");
