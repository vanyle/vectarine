use std::sync::Arc;

use glow::HasContext;

use crate::graphics::gltexture::ImageAntialiasing;

pub struct Framebuffer {
    id: glow::Framebuffer,
    // We store both color and stencil as texture for potential post-processing. This is more convenient than renderbuffers.
    color_tex: glow::NativeTexture,
    depth_stencil_tex: glow::NativeTexture,
    width: u32,
    height: u32,
    gl: Arc<glow::Context>,
}

impl Framebuffer {
    pub fn new_rgba(
        gl: &Arc<glow::Context>,
        width: u32,
        height: u32,
        filter: ImageAntialiasing,
    ) -> Self {
        unsafe {
            let id = gl.create_framebuffer().expect("Cannot create framebuffer");
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(id));

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                None,
                0,
            );

            let color_tex = gl.create_texture().expect("Cannot create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(color_tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );

            let gl_filter = filter.to_tex_parameter();
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, gl_filter);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, gl_filter);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(color_tex),
                0,
            );

            // Depth and stencil buffer attachments
            let depth_stencil_tex = gl.create_texture().expect("Cannot create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(depth_stencil_tex));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::DEPTH24_STENCIL8 as i32,
                width as i32,
                height as i32,
                0,
                glow::DEPTH_STENCIL,
                glow::UNSIGNED_INT_24_8,
                glow::PixelUnpackData::Slice(None),
            );

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::DEPTH_STENCIL_ATTACHMENT,
                glow::TEXTURE_2D,
                Some(depth_stencil_tex),
                0,
            );

            let status = gl.check_framebuffer_status(id.0.get());
            if status != glow::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer is not complete: {status}");
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            Self {
                id,
                width,
                height,
                gl: gl.clone(),
                color_tex,
                depth_stencil_tex,
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Bind the framebuffer, execute the closure, then unbind the framebuffer.
    /// The viewport is adjusted to match the framebuffer size during the execution of the closure.
    /// This means that any rendering done in the closure will be rendered to the framebuffer.
    pub fn using(&self, f: impl FnOnce()) {
        let mut viewport = [0, 0, 0, 0];
        unsafe {
            // Store current viewport
            self.gl
                .get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);
            let gl = self.gl.as_ref();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.id));
            gl.viewport(0, 0, self.width as i32, self.height as i32);
        }
        f();
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            // Restore previous viewport
            self.gl
                .viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
        }
    }

    pub fn bind_color_texture(&self, slot: u32) {
        unsafe {
            let gl = self.gl.as_ref();
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.color_tex));
        }
    }
    pub fn bind_depth_stencil_texture(&self, slot: u32) {
        unsafe {
            let gl = self.gl.as_ref();
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.depth_stencil_tex));
        }
    }

    pub fn id(&self) -> glow::NativeFramebuffer {
        self.id
    }

    pub fn color_texture_id(&self) -> glow::NativeTexture {
        self.color_tex
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.color_tex);
            self.gl.delete_texture(self.depth_stencil_tex);
            self.gl.delete_framebuffer(self.id);
        }
    }
}
