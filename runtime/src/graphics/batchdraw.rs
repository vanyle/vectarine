use std::{sync::Arc, time::Instant};

use crate::{
    game_resource::{
        font_resource::FontRenderingData, shader_resource::ShaderResource, ResourceId, ResourceManager
    }, graphics::{
        glbuffer::{BufferUsageHint, SharedGPUCPUBuffer},
        gldraw::DrawingTarget,
        glframebuffer::Framebuffer,
        glprogram::GLProgram,
        gltexture::Texture,
        gltypes::{DataLayout, GLTypes, UsageHint},
        gluniforms::{UniformValue, Uniforms},
        shadersources::{
            COLOR_FRAG_SHADER_SOURCE, COLOR_VERTEX_SHADER_SOURCE, FONT_FRAG_SHADER_SOURCE,
            FONT_VERTEX_SHADER_SOURCE, TEX_FRAG_SHADER_SOURCE, TEX_VERTEX_SHADER_SOURCE,
        },
        shape::Quad,
    }, io::IoEnvState, lua_env::lua_vec2::Vec2
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BatchShader {
    Color,
    Texture,
    Font,
    Custom(ResourceId), // Id of the custom shader
}

/// A simple structure to get quickly start drawing shapes.
/// Batches OpenGL calls together when possible.
/// Designed for immediate drawing
pub struct BatchDraw2d {
    color_program: GLProgram,
    texture_program: GLProgram,
    text_program: GLProgram,
    aspect_ratio: f32,

    vertex_data: Vec<(SharedGPUCPUBuffer, Uniforms, BatchShader)>,
    pub drawing_target: DrawingTarget,
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

    pub fn draw(&mut self, resources: &ResourceManager, auto_flush: bool) {
        // This is probably a dubious optimization, it needs to be benchmarked.
        let hint = if auto_flush {
            BufferUsageHint::StreamDraw
        } else {
            BufferUsageHint::StaticDraw
        };

        for (vertex, uniforms, shader) in &mut self.vertex_data {
            let draw = |vertex: &mut SharedGPUCPUBuffer, program, uniforms| {
                self.drawing_target.draw(
                    vertex.send_to_gpu_with_usage(self.drawing_target.gl(), &hint),
                    program,
                    uniforms,
                );
            };

            match shader {
                BatchShader::Color => draw(vertex, &self.color_program, uniforms),
                BatchShader::Texture => draw(vertex, &self.texture_program, uniforms),
                BatchShader::Font => draw(vertex, &self.text_program, uniforms),
                BatchShader::Custom(id) => {
                    let shader = resources.get_by_id::<ShaderResource>(id.to_owned());
                    let Ok(shader) = shader else {
                        continue;
                    };
                    let shader = &shader.shader;
                    let shader = shader.borrow();
                    let Some(shader) = shader.as_ref() else {
                        continue;
                    };
                    draw(vertex, &shader.shader, uniforms);
                    continue;
                }
            };
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
        shader_to_use: BatchShader,
    ) {
        let last_item = self.vertex_data.last_mut();
        let Some(last_item) = last_item else {
            self.add_to_batch_as_new_entry(vertices, indices, uniforms, shader_to_use);
            return;
        };
        let (last_vertex_buffer, last_uniforms, last_shader) = last_item;
        // Merging is not possible if the uniforms are not the same / the shader is different.
        if *last_shader != shader_to_use || !last_uniforms.similar(&uniforms) {
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
        shader_to_use: BatchShader,
    ) {
        let layout = (match shader_to_use {
            BatchShader::Color => &self.color_program,
            BatchShader::Texture => &self.texture_program,
            BatchShader::Font => &self.text_program,
            BatchShader::Custom(_) => {
                &self.texture_program // Custom shaders have the same layout as texture shaders
            }
        })
        .vertex_layout
        .clone();

        self.vertex_data.push((
            SharedGPUCPUBuffer::from_data(layout, vertices, indices),
            uniforms,
            shader_to_use,
        ));
    }

    pub fn draw_polygon(&mut self, points: Vec<Vec2>, color: [f32; 4]) {
        #[rustfmt::skip]
        let vertices: Vec<f32> = points.iter().flat_map(|p| {
            vec![
                p.x, p.y, 0.0, // position
                color[0], color[1], color[2], color[3], // color
            ]
        }).collect();

        // Triangulate the polygon using a triangle fan
        let mut indices: Vec<u32> = Vec::with_capacity((points.len() - 2) * 3);
        for i in 1..(points.len() - 1) {
            indices.push(0);
            indices.push(i as u32);
            indices.push((i + 1) as u32);
        }

        self.add_to_batch_by_trying_to_merge(
            &vertices,
            &indices,
            Uniforms::new(),
            BatchShader::Color,
        );
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

        self.add_to_batch_by_trying_to_merge(
            &vertices,
            &INDICES_FOR_QUAD,
            Uniforms::new(),
            BatchShader::Color,
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
            BatchShader::Color,
        );
    }

    pub fn draw_image(&mut self, x: f32, y: f32, width: f32, height: f32, texture: &Arc<Texture>) {
        let uv_pos = Vec2::new(0.0, 0.0);
        let uv_size = Vec2::new(1.0, 1.0);
        self.draw_image_part(make_rect(x, y, width, height), texture, uv_pos, uv_size);
    }

    #[rustfmt::skip]
    pub fn draw_image_part(
        &mut self, pos_size: Quad, texture: &Arc<Texture>, uv_pos: Vec2, uv_size: Vec2,
    ) {
        let uv_x1 = uv_pos.x;
        let uv_y1 = uv_pos.y;
        let uv_x2 = uv_pos.x + uv_size.x;
        let uv_y2 = uv_pos.y + uv_size.y;

        #[rustfmt::skip]
        let vertices: [f32; 4 * 5] = [
            // positions       // tex coords
            pos_size.p1.x, pos_size.p1.y, 0.0, uv_x1, uv_y2, // bottom left
            pos_size.p2.x, pos_size.p2.y, 0.0, uv_x2, uv_y2, // bottom right
            pos_size.p3.x, pos_size.p3.y, 0.0, uv_x2, uv_y1, // top right
            pos_size.p4.x, pos_size.p4.y, 0.0, uv_x1, uv_y1, // top left
        ];

        let mut uniforms = Uniforms::new();
        
        uniforms.add("tex", UniformValue::Sampler2D(texture.id()));

        self.add_to_batch_by_trying_to_merge(&vertices, &INDICES_FOR_QUAD, uniforms, BatchShader::Texture);
    }

    pub fn draw_canvas(
        &mut self,
        pos: Vec2,
        size: Vec2,
        canvas: &Framebuffer,
        custom_shader: Option<ResourceId>,
        env: &IoEnvState,
    ) {
        self.draw_canvas_part(
            make_rect(pos.x, pos.y, size.x, size.y),
            canvas,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            custom_shader,
            env,
        );
    }

    #[rustfmt::skip]
    pub fn draw_canvas_part(
        &mut self, pos_size: Quad, canvas: &Framebuffer, uv_pos: Vec2, uv_size: Vec2,
        custom_shader: Option<ResourceId>, env: &IoEnvState
    ) {
        let uv_x1 = uv_pos.x;
        let uv_y1 = uv_pos.y;
        let uv_x2 = uv_pos.x + uv_size.x;
        let uv_y2 = uv_pos.y + uv_size.y;

        // Weird that we need to flip the y coordinates in canvas, but not image.
        #[rustfmt::skip]
        let vertices: [f32; 4 * 5] = [
            // positions       // tex coords
            pos_size.p4.x, pos_size.p4.y, 0.0, uv_x1, uv_y2, // bottom left
            pos_size.p3.x, pos_size.p3.y, 0.0, uv_x2, uv_y2, // bottom right
            pos_size.p2.x, pos_size.p2.y, 0.0, uv_x2, uv_y1, // top right
            pos_size.p1.x, pos_size.p1.y, 0.0, uv_x1, uv_y1, // top left
        ];

        let mut uniforms = Uniforms::new();
        // Add uniforms to replicate shader toy style
        uniforms.add("tex", UniformValue::Sampler2D(canvas.color_texture_id()));
        let elapsed = Instant::now() - env.start_time;
        uniforms.add("iTime", UniformValue::Float(elapsed.as_secs_f32()));

        let shader_to_use = if let Some(id) = custom_shader {
            BatchShader::Custom(id)
        } else {
            BatchShader::Texture
        };
        self.add_to_batch_by_trying_to_merge(&vertices, &INDICES_FOR_QUAD, uniforms, shader_to_use);
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
            UniformValue::Sampler2D(font_resource.font_atlas.id()),
        );
        uniforms.add("text_color", UniformValue::Vec4(color));
        self.add_to_batch_by_trying_to_merge(&vertices, &indices, uniforms, BatchShader::Font);
    }

    pub fn flush(&mut self) {
        self.vertex_data.clear();
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        self.drawing_target.clear(r, g, b, a);
    }
}

const INDICES_FOR_QUAD: [u32; 6] = [
    0, 1, 2, // first triangle
    2, 3, 0, // second triangle
];

pub fn make_rect(x: f32, y: f32, width: f32, height: f32) -> Quad {
    Quad {
        p1: Vec2::new(x, y),
        p2: Vec2::new(x + width, y),
        p3: Vec2::new(x + width, y + height),
        p4: Vec2::new(x, y + height),
    }
}
