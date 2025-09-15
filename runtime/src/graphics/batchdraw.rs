use std::sync::Arc;

use crate::{
    graphics::{
        glbuffer::{BufferUsageHint, SharedGPUCPUBuffer},
        gldraw::DrawingTarget,
        glprogram::GLProgram,
        gltexture::Texture,
        gltypes::{DataLayout, GLTypes, UsageHint},
        gluniforms::{UniformValue, Uniforms},
        shadersources::{
            COLOR_FRAG_SHADER_SOURCE, COLOR_VERTEX_SHADER_SOURCE, FONT_FRAG_SHADER_SOURCE,
            FONT_VERTEX_SHADER_SOURCE, TEX_FRAG_SHADER_SOURCE, TEX_VERTEX_SHADER_SOURCE,
        },
    },
    helpers::game_resource::font_resource::FontRenderingData,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DefaultShader {
    Color,
    Texture,
    Font,
}

/// A simple structure to get quickly start drawing shapes.
/// Batches OpenGL calls together when possible.
/// Designed for immediate drawing
pub struct BatchDraw2d {
    color_program: GLProgram,
    texture_program: GLProgram,
    text_program: GLProgram,
    aspect_ratio: f32,

    vertex_data: Vec<(SharedGPUCPUBuffer, Uniforms, DefaultShader)>,
    drawing_target: DrawingTarget,
}

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

        let mut text_program =
            GLProgram::from_source(gl, FONT_VERTEX_SHADER_SOURCE, FONT_FRAG_SHADER_SOURCE)?;
        let mut layout = DataLayout::new();
        layout
            .add_field("in_vert", GLTypes::Vec2, Some(UsageHint::Position))
            .add_field("in_uv", GLTypes::Vec2, Some(UsageHint::TexCoord));
        text_program.vertex_layout = layout;

        let drawing_target = DrawingTarget::new(gl);

        Ok(Self {
            color_program,
            texture_program,
            text_program,
            vertex_data: Vec::new(),
            aspect_ratio: 1.0,
            drawing_target,
        })
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn draw(&mut self, auto_flush: bool) {
        for (vertex, uniforms, shader) in &mut self.vertex_data {
            let program = match shader {
                DefaultShader::Color => &self.color_program,
                DefaultShader::Texture => &self.texture_program,
                DefaultShader::Font => &self.text_program,
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
        shader_to_use: DefaultShader,
    ) {
        let last_item = self.vertex_data.last_mut();
        let Some(last_item) = last_item else {
            self.add_to_batch_as_new_entry(vertices, indices, uniforms, shader_to_use);
            return;
        };
        let (last_vertex_buffer, last_uniforms, last_is_textured) = last_item;
        // Merging is not possible if the uniforms are not the same / the shader is different.
        if *last_is_textured != shader_to_use || !last_uniforms.similar(&uniforms) {
            self.add_to_batch_as_new_entry(vertices, indices, uniforms, shader_to_use);
            return;
        }

        last_vertex_buffer.append_from(vertices, indices);
    }

    fn add_to_batch_as_new_entry(
        &mut self,
        vertices: &[f32],
        indices: &[u32],
        uniforms: Uniforms,
        shader_to_use: DefaultShader,
    ) {
        let layout = (match shader_to_use {
            DefaultShader::Color => &self.color_program,
            DefaultShader::Texture => &self.texture_program,
            DefaultShader::Font => &self.text_program,
        })
        .vertex_layout
        .clone();

        self.vertex_data.push((
            SharedGPUCPUBuffer::from_data(layout, vertices, indices),
            uniforms,
            shader_to_use,
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

        self.add_to_batch_by_trying_to_merge(
            &vertices,
            &indices,
            Uniforms::new(),
            DefaultShader::Color,
        );
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
            let vx = x + (radius * theta.cos()) / self.aspect_ratio;
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

        self.add_to_batch_by_trying_to_merge(
            &vertices,
            &indices,
            Uniforms::new(),
            DefaultShader::Color,
        );
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
        self.add_to_batch_by_trying_to_merge(&vertices, &indices, uniforms, DefaultShader::Texture);
    }

    pub fn draw_text(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        color: [f32; 4],
        font_size: f32,
        font_resource: &FontRenderingData,
    ) {
        let scale = font_size / font_resource.font_size;
        let mut vertices = Vec::<f32>::new();
        let mut indices = Vec::<u32>::new();
        let mut x_pos = 0.0;
        let mut y_pos = 0.0;

        for c in text.chars() {
            if let Some(char_info) = font_resource.font_cache.get(&c) {
                let bounds = char_info.metrics.bounds.scale(scale);
                let x0 = x + (x_pos + bounds.xmin) / self.aspect_ratio;
                let y0 = y + y_pos + bounds.ymin;
                let x1 = x0 + bounds.width / self.aspect_ratio;
                let y1 = y0 + bounds.height;

                x_pos += char_info.metrics.advance_width * scale;
                y_pos += char_info.metrics.advance_height * scale;

                // Use the stored atlas coordinates instead of calculating from metrics
                let s0 = char_info.atlas_x;
                let t0 = char_info.atlas_y;
                let s1 = char_info.atlas_x + char_info.atlas_width;
                let t1 = char_info.atlas_y + char_info.atlas_height + 0.04;

                let s = &[
                    // positions       // tex coords
                    x0, y0, s0, t1, // bottom left
                    x1, y0, s1, t1, // bottom right
                    x1, y1, s1, t0, // top right
                    x0, y1, s0, t0, // top left
                ];

                vertices.extend_from_slice(s);

                let base_index = (vertices.len() / 4 - 4) as u32; // Each vertex has 4 components

                indices.extend_from_slice(&[
                    base_index,
                    base_index + 1,
                    base_index + 2, // first triangle
                    base_index + 2,
                    base_index + 3,
                    base_index, // second triangle
                ]);
            }
        }

        let mut uniforms = Uniforms::new();
        uniforms.add(
            "tex",
            UniformValue::Sampler2D(font_resource.font_atlas.clone()),
        );
        uniforms.add("text_color", UniformValue::Vec4(color));
        self.add_to_batch_by_trying_to_merge(&vertices, &indices, uniforms, DefaultShader::Font);
    }

    pub fn flush(&mut self) {
        self.vertex_data.clear();
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        self.drawing_target.clear(r, g, b, a);
    }
}
