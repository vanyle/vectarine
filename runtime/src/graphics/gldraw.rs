use std::sync::Arc;

use glow::HasContext;

use crate::graphics::{glbuffer::GpuVertexData, glprogram::GLProgram, gluniforms::Uniforms};

#[derive(Debug, Clone, Copy)]
pub struct DrawParams {
    pub depth_test: bool,
    pub blend: bool,
    pub cull_face: bool,
}

/// Represents a thing that can be drawn to.
pub struct DrawingTarget {
    gl: Arc<glow::Context>,
    pub current_param_state: DrawParams,
}

impl DrawingTarget {
    pub fn new(gl: &Arc<glow::Context>) -> Self {
        Self {
            gl: gl.clone(),
            current_param_state: DrawParams {
                depth_test: true,
                blend: false,
                cull_face: false,
            },
        }
    }

    pub fn gl(&self) -> &Arc<glow::Context> {
        &self.gl
    }

    pub fn draw(
        &self,
        vertex_buffer: &GpuVertexData,
        program: &GLProgram,
        uniforms: &Uniforms,
        // draw_params: &DrawParams,
    ) {
        // ...
        let gl = self.gl.as_ref();
        program.use_program();
        program.set_uniforms(uniforms);
        vertex_buffer.bind_for_drawing();

        let points = vertex_buffer.drawn_point_count as i32;
        unsafe {
            gl.draw_elements(glow::TRIANGLES, points, glow::UNSIGNED_INT, 0);
        }
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        let gl = self.gl.as_ref();
        unsafe {
            gl.clear_color(r, g, b, a);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }
}
