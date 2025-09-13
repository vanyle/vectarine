use std::sync::Arc;

use glow::{HasContext, PixelUnpackData};

/// Represents a texture on the GPU
#[derive(Debug, Clone)]
pub struct Texture {
    tex: glow::NativeTexture,
    width: u32,
    height: u32,
    gl: Arc<glow::Context>,
}

impl Texture {
    /// Create a new RGBA texture
    pub fn new_rgba(gl: &Arc<glow::Context>, data: &[u8], width: u32, height: u32) -> Arc<Self> {
        unsafe {
            let glref = gl.as_ref();
            let tex = glref.create_texture().expect("Cannot create texture");

            glref.bind_texture(glow::TEXTURE_2D, Some(tex));
            glref.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            glref.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            // set texture filtering parameters
            glref.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            glref.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            glref.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(data)),
            );

            Arc::new(Self {
                tex,
                width,
                height,
                gl: gl.clone(),
            })
        }
    }

    pub fn bind(self: &Arc<Self>, slot: u32) {
        unsafe {
            let gl = self.gl.as_ref();
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
        }
    }

    pub fn width(self: &Arc<Self>) -> u32 {
        self.width
    }

    pub fn height(self: &Arc<Self>) -> u32 {
        self.height
    }
}
