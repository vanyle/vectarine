use std::{cell::RefCell, sync::Arc};

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
    draw_call_counter: RefCell<usize>,
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
            draw_call_counter: RefCell::new(0),
        }
    }

    pub fn gl(&self) -> &Arc<glow::Context> {
        &self.gl
    }

    pub fn draw(&self, vertex_buffer: &GpuVertexData, program: &GLProgram, uniforms: &Uniforms) {
        // Note: We don't handle DrawParams (glEnable(something)) here for now.
        let gl = self.gl.as_ref();
        program.use_program();
        program.set_uniforms(uniforms);
        vertex_buffer.bind_for_drawing();

        *self.draw_call_counter.borrow_mut() += 1;
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

    pub fn get_draw_call_counter(&self) -> usize {
        *self.draw_call_counter.borrow()
    }

    pub fn reset_draw_call_counter(&self) {
        *self.draw_call_counter.borrow_mut() = 0;
    }

    pub fn enable_multisampling(&self) {
        unsafe {
            self.gl.as_ref().enable(glow::BLEND);
            self.gl
                .as_ref()
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.as_ref().enable(glow::MULTISAMPLE);
        }
    }
}
