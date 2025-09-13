use std::sync::Arc;

use crate::graphics::{
    glbuffer::{BufferUsageHint, GpuVertexData},
    gldraw::DrawingTarget,
    glprogram::GLProgram,
    gltypes::{DataLayout, GLTypes, UsageHint},
    gluniforms::Uniforms,
};

/// A simple structure to get quickly start drawing shapes.
/// Batches OpenGL calls together when possible.
/// Designed for immediate drawing
pub struct BatchDraw2d {
    color_program: GLProgram,
    texture_program: GLProgram,

    vertex_data: Vec<(GpuVertexData, Uniforms, bool)>, // bool = is textured?
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

        let texture_program =
            GLProgram::from_source(gl, TEX_VERTEX_SHADER_SOURCE, TEX_FRAG_SHADER_SOURCE)?;
        let mut layout = DataLayout::new();
        layout
            .add_field("in_vert", GLTypes::Vec3, Some(UsageHint::Position))
            .add_field("in_uv", GLTypes::Vec2, Some(UsageHint::TexCoord));

        let drawing_target = DrawingTarget::new(gl);

        Ok(Self {
            color_program,
            texture_program,
            vertex_data: Vec::new(),
            drawing_target,
        })
    }

    pub fn draw(&mut self, auto_flush: bool) {
        for (vertex, uniforms, is_textured) in &self.vertex_data {
            let program = if *is_textured {
                &self.texture_program
            } else {
                &self.color_program
            };

            self.drawing_target.draw(vertex, program, uniforms);
        }
        if auto_flush {
            self.flush();
        }
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

        let mut vertex_buffer = GpuVertexData::new(self.drawing_target.gl());
        vertex_buffer.apply_layout(self.color_program.vertex_layout.clone());
        vertex_buffer
            .set_data_with_usage(&vertices, &indices, BufferUsageHint::StreamDraw)
            .expect("The data shape matches the layout.");

        let uniforms = Uniforms::new();
        self.vertex_data.push((vertex_buffer, uniforms, false));
    }

    pub fn flush(&mut self) {
        self.vertex_data.clear();
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        self.drawing_target.clear(r, g, b, a);
    }
}
