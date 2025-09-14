use std::sync::Arc;

use crate::graphics::{
    glbuffer::{BufferUsageHint, SharedGPUCPUBuffer},
    gldraw::DrawingTarget,
    glprogram::GLProgram,
    gltexture::Texture,
    gltypes::{DataLayout, GLTypes, UsageHint},
    gluniforms::{UniformValue, Uniforms},
};

/// A simple structure to get quickly start drawing shapes.
/// Batches OpenGL calls together when possible.
/// Designed for immediate drawing
pub struct BatchDraw2d {
    color_program: GLProgram,
    texture_program: GLProgram,

    vertex_data: Vec<(SharedGPUCPUBuffer, Uniforms, bool)>, // bool = is textured?
    drawing_target: DrawingTarget,
}

const COLOR_VERTEX_SHADER_SOURCE: &str = r#"
    layout (location = 0) in vec3 in_vert;
    layout (location = 1) in vec4 in_color;
    out vec4 color;
    void main() {
        color = in_color;
        gl_Position = vec4(in_vert.xyz, 1.0);
    }"#;

const COLOR_FRAG_SHADER_SOURCE: &str = r#"precision mediump float;
    in vec4 color;
    out vec4 frag_color;
    void main() {
        frag_color = color;
    }"#;

const TEX_VERTEX_SHADER_SOURCE: &str = r#"
    layout (location = 0) in vec3 in_vert;
    layout (location = 1) in vec2 in_uv;
    out vec2 uv;
    void main() {
        uv = in_uv;
        gl_Position = vec4(in_vert.xyz, 1.0);
    }"#;

const TEX_FRAG_SHADER_SOURCE: &str = r#"precision mediump float;
    in vec2 uv;
    uniform sampler2D tex;
    out vec4 frag_color;
    void main() {
        frag_color = texture(tex, uv);
    }"#;

impl BatchDraw2d {
    // Create a new batch for drawing on the current window.
    pub fn new(gl: &Arc<glow::Context>) -> Result<Self, String> {
        let mut color_program =
            GLProgram::from_source(gl, COLOR_VERTEX_SHADER_SOURCE, COLOR_FRAG_SHADER_SOURCE)?;
        let mut layout = DataLayout::new();
        layout
            .add_field("in_vert", GLTypes::Vec3, Some(UsageHint::Position))
            .add_field("in_color", GLTypes::Vec4, Some(UsageHint::Color));
        color_program.vertex_layout = layout;

        let mut texture_program =
            GLProgram::from_source(gl, TEX_VERTEX_SHADER_SOURCE, TEX_FRAG_SHADER_SOURCE)?;
        let mut layout = DataLayout::new();
        layout
            .add_field("in_vert", GLTypes::Vec3, Some(UsageHint::Position))
            .add_field("in_uv", GLTypes::Vec2, Some(UsageHint::TexCoord));
        texture_program.vertex_layout = layout;

        let drawing_target = DrawingTarget::new(gl);

        Ok(Self {
            color_program,
            texture_program,
            vertex_data: Vec::new(),
            drawing_target,
        })
    }

    pub fn draw(&mut self, auto_flush: bool) {
        for (vertex, uniforms, is_textured) in &mut self.vertex_data {
            let program = if *is_textured {
                &self.texture_program
            } else {
                &self.color_program
            };

            // This is probably a dubious optimization, it needs to be benchmarked.
            let hint = if auto_flush {
                BufferUsageHint::StreamDraw
            } else {
                BufferUsageHint::StaticDraw
            };

            self.drawing_target.draw(
                vertex.send_to_gpu_with_usage(self.drawing_target.gl(), hint),
                program,
                uniforms,
            );
        }
        if auto_flush {
            self.flush();
        }
    }

    fn add_to_batch_by_trying_to_merge(
        &mut self,
        vertices: &[f32],
        indices: &[u32],
        uniforms: Uniforms,
        is_textured: bool,
    ) {
        let last_item = self.vertex_data.last_mut();
        let Some(last_item) = last_item else {
            self.add_to_batch_as_new_entry(vertices, indices, uniforms, is_textured);
            return;
        };
        let (last_vertex_buffer, last_uniforms, last_is_textured) = last_item;
        // Merging is not possible if the uniforms are not the same / the shader is different.
        if *last_is_textured != is_textured || !last_uniforms.similar(&uniforms) {
            self.add_to_batch_as_new_entry(vertices, indices, uniforms, is_textured);
            return;
        }

        last_vertex_buffer.append_from(vertices, indices);
    }

    fn add_to_batch_as_new_entry(
        &mut self,
        vertices: &[f32],
        indices: &[u32],
        uniforms: Uniforms,
        is_textured: bool,
    ) {
        let layout = if is_textured {
            self.texture_program.vertex_layout.clone()
        } else {
            self.color_program.vertex_layout.clone()
        };
        self.vertex_data.push((
            SharedGPUCPUBuffer::from_data(layout, vertices, indices),
            uniforms,
            is_textured,
        ));
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        #[rustfmt::skip]
        let vertices: [f32; 4 * 7] = [
            // positions       // colors
            x, y, 0.0, color[0], color[1], color[2], color[3], // bottom left
            x + width, y, 0.0, color[0], color[1], color[2], color[3], // bottom right
            x + width, y + height, 0.0, color[0], color[1], color[2], color[3], // top right
            x, y + height, 0.0, color[0], color[1], color[2], color[3], // top left
        ];

        let indices: [u32; 6] = [
            0, 1, 2, // first triangle
            2, 3, 0, // second triangle
        ];

        self.add_to_batch_by_trying_to_merge(&vertices, &indices, Uniforms::new(), false);
    }

    pub fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: [f32; 4]) {
        const CIRCLE_SEGMENTS: usize = 32;
        let mut vertices: Vec<f32> = Vec::with_capacity((CIRCLE_SEGMENTS + 1) * 7);
        let mut indices: Vec<u32> = Vec::with_capacity(CIRCLE_SEGMENTS * 3);

        // Center vertex
        vertices.push(x);
        vertices.push(y);
        vertices.push(0.0);
        vertices.extend_from_slice(&color);

        for i in 0..=CIRCLE_SEGMENTS {
            let theta = (i as f32 / CIRCLE_SEGMENTS as f32) * std::f32::consts::TAU;
            let vx = x + radius * theta.cos();
            let vy = y + radius * theta.sin();
            vertices.push(vx);
            vertices.push(vy);
            vertices.push(0.0);
            vertices.extend_from_slice(&color);

            if i < CIRCLE_SEGMENTS {
                indices.push(0);
                indices.push(i as u32 + 1);
                indices.push(i as u32 + 2);
            }
        }

        self.add_to_batch_by_trying_to_merge(&vertices, &indices, Uniforms::new(), false);
    }

    pub fn draw_image(&mut self, x: f32, y: f32, width: f32, height: f32, texture: &Arc<Texture>) {
        #[rustfmt::skip]
        let vertices: [f32; 4 * 5] = [
            // positions       // tex coords
            x, y, 0.0, 0.0, 0.0, // bottom left
            x + width, y, 0.0, 1.0, 0.0, // bottom right
            x + width, y + height, 0.0, 1.0, 1.0, // top right
            x, y + height, 0.0, 0.0, 1.0, // top left
        ];

        let indices: [u32; 6] = [
            0, 1, 2, // first triangle
            2, 3, 0, // second triangle
        ];

        let mut uniforms = Uniforms::new();
        uniforms.add("tex", UniformValue::Sampler2D(texture.clone()));
        self.add_to_batch_by_trying_to_merge(&vertices, &indices, uniforms, true);
    }

    pub fn flush(&mut self) {
        self.vertex_data.clear();
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        self.drawing_target.clear(r, g, b, a);
    }
}
