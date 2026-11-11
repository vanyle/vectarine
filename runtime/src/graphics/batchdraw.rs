use std::{collections::HashMap, sync::Arc};

use vectarine_plugin_sdk::wgpu;

use crate::{
    console::print_warn,
    game_resource::{
        ResourceId, ResourceManager, font_resource::FontRenderingData,
        shader_resource::ShaderResource,
    },
    graphics::{
        affinetransform::AffineTransform,
        framebuffer::Framebuffer,
        gfx::Gfx,
        pipeline::{BuiltinShader, VertexFormat},
        shape::Quad,
        texture::Texture,
    },
    lua_env::lua_vec2::Vec2,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BatchShader {
    Color,
    Texture,
    Font,
    Custom(ResourceId), // Id of the custom shader
}

impl BatchShader {
    fn vertex_format(self) -> VertexFormat {
        match self {
            BatchShader::Color => VertexFormat::PositionColor,
            BatchShader::Texture | BatchShader::Font => VertexFormat::PositionUvColor,
            BatchShader::Custom(_) => VertexFormat::PositionUv,
        }
    }
}

/// A custom shader used to draw a canvas, with the values of its uniforms for this drawing.
pub struct CanvasShader {
    pub id: ResourceId,
    pub uniforms: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq)]
struct BatchTexture {
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
}

impl BatchTexture {
    fn from_texture(texture: &Texture) -> Self {
        Self {
            bind_group: texture.bind_group().clone(),
            texture: texture.raw().clone(),
        }
    }
}

/// Drawings that can be sent to the graphics card in one call: same shader, same texture, same uniforms.
struct BatchEntry {
    shader: BatchShader,
    texture: Option<BatchTexture>,
    uniforms: Option<Vec<u8>>,
    vertices: Vec<f32>,
    indices: Vec<u32>,
}

/// A simple structure to get quickly start drawing shapes.
/// Batches draw calls together when possible.
/// Designed for immediate drawing
pub struct BatchDraw2d {
    aspect_ratio: f32,

    pub affine_transform: AffineTransform,

    entries: Vec<BatchEntry>,
    gfx: Gfx,

    vertex_staging: Vec<f32>,
    index_staging: Vec<u32>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}

impl BatchDraw2d {
    // Create a new batch for drawing on the current target of `gfx`.
    pub fn new(gfx: &Gfx) -> Result<Self, String> {
        Ok(Self {
            entries: Vec::new(),
            aspect_ratio: 1.0,
            affine_transform: AffineTransform::identity(),
            gfx: gfx.clone(),
            vertex_staging: Vec::new(),
            index_staging: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
        })
    }

    pub fn gfx(&self) -> &Gfx {
        &self.gfx
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn draw(&mut self, resources: &ResourceManager, auto_flush: bool) {
        if !self.entries.is_empty() {
            self.send_to_gpu(resources);
        }
        if auto_flush {
            self.flush();
        }
    }

    fn send_to_gpu(&mut self, resources: &ResourceManager) {
        let gfx = self.gfx.clone();
        let gpu = gfx.gpu();
        let flip_y = gfx.flip_y();

        // All the entries share a vertex buffer and an index buffer, each has its own range inside.
        self.vertex_staging.clear();
        self.index_staging.clear();
        let mut ranges = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            let vertex_start = self.vertex_staging.len();
            let index_start = self.index_staging.len();
            self.vertex_staging.extend_from_slice(&entry.vertices);
            self.index_staging.extend_from_slice(&entry.indices);
            if flip_y {
                let stride = entry.shader.vertex_format().floats_per_vertex();
                for vertex in self.vertex_staging[vertex_start..].chunks_exact_mut(stride) {
                    vertex[1] = -vertex[1];
                }
            }
            ranges.push((
                (vertex_start * size_of::<f32>()) as u64
                    ..(self.vertex_staging.len() * size_of::<f32>()) as u64,
                index_start as u32..self.index_staging.len() as u32,
            ));
        }

        let vertex_buffer = write_buffer(
            &gfx,
            &mut self.vertex_buffer,
            wgpu::BufferUsages::VERTEX,
            bytemuck::cast_slice(&self.vertex_staging),
        );
        let index_buffer = write_buffer(
            &gfx,
            &mut self.index_buffer,
            wgpu::BufferUsages::INDEX,
            bytemuck::cast_slice(&self.index_staging),
        );

        let entries = &self.entries;
        gfx.render(|pass, info| {
            let mut uniform_slots: HashMap<ResourceId, usize> = HashMap::new();
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for (entry, (vertex_range, index_range)) in entries.iter().zip(ranges) {
                if let Some(texture) = &entry.texture
                    && texture.texture == info.texture
                {
                    print_warn("A canvas cannot be drawn on itself".to_string());
                    continue;
                }

                match entry.shader {
                    BatchShader::Color => {
                        pass.set_pipeline(&gfx.builtin_pipeline(BuiltinShader::Color, info.target));
                    }
                    BatchShader::Texture => {
                        pass.set_pipeline(
                            &gfx.builtin_pipeline(BuiltinShader::Texture, info.target),
                        );
                    }
                    BatchShader::Font => {
                        pass.set_pipeline(&gfx.builtin_pipeline(BuiltinShader::Font, info.target));
                    }
                    BatchShader::Custom(id) => {
                        let Ok(shader) = resources.get_by_id::<ShaderResource>(id) else {
                            continue;
                        };
                        let shader = shader.shader.borrow();
                        let Some(shader) = shader.as_ref() else {
                            continue;
                        };
                        pass.set_pipeline(&shader.pipeline(gpu, info.target));

                        let slot = uniform_slots.entry(id).or_default();
                        let uniforms = match &entry.uniforms {
                            Some(uniforms) => shader.uniform_bind_group(gpu, *slot, uniforms),
                            None => None,
                        };
                        // The shader was reloaded since the drawing was asked: its current uniforms are used.
                        let uniforms = uniforms.or_else(|| {
                            let current = shader.uniform_snapshot(0.0)?;
                            shader.uniform_bind_group(gpu, *slot, &current)
                        });
                        if let Some(uniforms) = uniforms {
                            pass.set_bind_group(1, &uniforms, &[]);
                            *slot += 1;
                        }
                    }
                }
                if let Some(texture) = &entry.texture {
                    pass.set_bind_group(0, &texture.bind_group, &[]);
                }
                pass.set_vertex_buffer(0, vertex_buffer.slice(vertex_range));
                pass.draw_indexed(index_range, 0, 0..1);
                gfx.count_draw_call();
            }
        });
    }

    fn add_to_batch_by_trying_to_merge(
        &mut self,
        vertices: &[f32],
        indices: &[u32],
        shader: BatchShader,
        texture: Option<BatchTexture>,
        uniforms: Option<Vec<u8>>,
    ) {
        // Trying to draw empty data can lead to crashes.
        if vertices.is_empty() || indices.is_empty() {
            return;
        }

        // Merging is not possible if the uniforms are not the same / the shader is different.
        if let Some(last) = self.entries.last_mut()
            && last.shader == shader
            && last.texture == texture
            && last.uniforms == uniforms
        {
            let vertex_offset =
                (last.vertices.len() / shader.vertex_format().floats_per_vertex()) as u32;
            last.vertices.extend_from_slice(vertices);
            last.indices
                .extend(indices.iter().map(|i| i + vertex_offset));
            return;
        }

        self.entries.push(BatchEntry {
            shader,
            texture,
            uniforms,
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
        });
    }

    pub fn draw_polygon(&mut self, points: impl Iterator<Item = Vec2>, color: [f32; 4]) {
        let mut points_len = 0;
        #[rustfmt::skip]
        let vertices: Vec<f32> = points.flat_map(|p| {
            points_len += 1;
            let p = self.affine_transform.apply(&p);
            vec![
                p.x(), p.y(), // position
                color[0], color[1], color[2], color[3], // color
            ]
        }).collect();

        if points_len < 3 {
            return; // Not enough points to form a polygon
        }

        // Triangulate the polygon using a triangle fan
        let mut indices: Vec<u32> = Vec::with_capacity((points_len - 2) * 3);
        for i in 1..(points_len - 1) {
            indices.push(0);
            indices.push(i as u32);
            indices.push((i + 1) as u32);
        }

        self.add_to_batch_by_trying_to_merge(&vertices, &indices, BatchShader::Color, None, None);
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        let p = self.affine_transform.apply(&Vec2::new(x, y));
        let q = self
            .affine_transform
            .apply(&Vec2::new(x + width, y + height));

        #[rustfmt::skip]
        let vertices: [f32; 4 * 6] = [
            // positions       // colors
            p.x(), p.y(), color[0], color[1], color[2], color[3], // bottom left
            q.x(), p.y(), color[0], color[1], color[2], color[3], // bottom right
            q.x(), q.y(), color[0], color[1], color[2], color[3], // top right
            p.x(), q.y(), color[0], color[1], color[2], color[3], // top left
        ];

        self.add_to_batch_by_trying_to_merge(
            &vertices,
            &INDICES_FOR_QUAD,
            BatchShader::Color,
            None,
            None,
        );
    }

    #[inline]
    pub fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: [f32; 4]) {
        self.draw_ellipse(x, y, radius / self.aspect_ratio, radius, color);
    }

    pub fn draw_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        let circle_segment_count: usize = if (width.abs() + height.abs()) < 0.05 {
            32
        } else {
            64
        };

        let mut vertices: Vec<f32> = Vec::with_capacity((circle_segment_count + 1) * (2 + 4));
        let mut indices: Vec<u32> = Vec::with_capacity(circle_segment_count * 3);

        // Center vertex
        let p = self.affine_transform.apply(&Vec2::new(x, y));
        vertices.push(p.x());
        vertices.push(p.y());
        vertices.extend_from_slice(&color);

        for i in 0..=circle_segment_count {
            let theta = (i as f32 / circle_segment_count as f32) * std::f32::consts::TAU;
            let vx = x + (width * theta.cos());
            let vy = y + height * theta.sin();
            let p = self.affine_transform.apply(&Vec2::new(vx, vy));
            vertices.push(p.x());
            vertices.push(p.y());
            vertices.extend_from_slice(&color);

            if i < circle_segment_count {
                indices.push(0);
                indices.push(i as u32 + 1);
                indices.push(i as u32 + 2);
            }
        }

        self.add_to_batch_by_trying_to_merge(&vertices, &indices, BatchShader::Color, None, None);
    }

    pub fn draw_image(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        texture: &Arc<Texture>,
        color: [f32; 4],
    ) {
        let uv_pos = Vec2::new(0.0, 0.0);
        let uv_size = Vec2::new(1.0, 1.0);

        self.draw_image_part(
            make_rect(x, y, width, height),
            texture,
            uv_pos,
            uv_size,
            color,
        );
    }

    pub fn draw_image_part(
        &mut self,
        pos_size: Quad,
        texture: &Arc<Texture>,
        uv_pos: Vec2,
        uv_size: Vec2,
        color: [f32; 4],
    ) {
        let vertices = self.image_vertices(&pos_size, uv_pos, uv_size, color);
        self.add_to_batch_by_trying_to_merge(
            &vertices,
            &INDICES_FOR_QUAD,
            BatchShader::Texture,
            Some(BatchTexture::from_texture(texture)),
            None,
        );
    }

    #[rustfmt::skip]
    fn image_vertices(&self, pos_size: &Quad, uv_pos: Vec2, uv_size: Vec2, color: [f32; 4]) -> [f32; 4 * 8] {
        let uv_x1 = uv_pos.x();
        let uv_y1 = uv_pos.y();
        let uv_x2 = uv_pos.x() + uv_size.x();
        let uv_y2 = uv_pos.y() + uv_size.y();

        let p1 = self.affine_transform.apply(&pos_size.p1);
        let p2 = self.affine_transform.apply(&pos_size.p2);
        let p3 = self.affine_transform.apply(&pos_size.p3);
        let p4 = self.affine_transform.apply(&pos_size.p4);
        let [r, g, b, a] = color;

        [
            // positions    // tex coords  // tint
            p1.x(), p1.y(), uv_x1, uv_y2, r, g, b, a, // bottom left
            p2.x(), p2.y(), uv_x2, uv_y2, r, g, b, a, // bottom right
            p3.x(), p3.y(), uv_x2, uv_y1, r, g, b, a, // top right
            p4.x(), p4.y(), uv_x1, uv_y1, r, g, b, a, // top left
        ]
    }

    /// Like draw_image_part, but draws lots of images with the same texture and shader. Ideal for efficiently drawing many tiles.
    /// All draw calls already are batched by default, but this reduces allocation a bit by reducing append that can grow the array and cause reallocations.
    pub fn draw_images_part(
        &mut self,
        quads: &[Quad],
        texture: &Arc<Texture>,
        uv_pos_size: &[(Vec2, Vec2)],
        color: [f32; 4],
    ) {
        let vertices: Box<[f32]> = quads
            .iter()
            .zip(uv_pos_size)
            .flat_map(|(pos_size, (uv_pos, uv_size))| {
                self.image_vertices(pos_size, *uv_pos, *uv_size, color)
            })
            .collect::<Box<[f32]>>();

        let quad_count = quads.len().min(uv_pos_size.len());
        let indices: Vec<u32> = (0..quad_count as u32)
            .flat_map(|q| {
                let base = q * 4;
                INDICES_FOR_QUAD.iter().map(move |i| i + base)
            })
            .collect();

        self.add_to_batch_by_trying_to_merge(
            &vertices,
            &indices,
            BatchShader::Texture,
            Some(BatchTexture::from_texture(texture)),
            None,
        );
    }

    pub fn draw_canvas(
        &mut self,
        pos: Vec2,
        size: Vec2,
        canvas: &Framebuffer,
        custom_shader: Option<CanvasShader>,
    ) {
        self.draw_canvas_part(
            make_rect(pos.x(), pos.y(), size.x(), size.y()),
            canvas,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            custom_shader,
        );
    }

    pub fn draw_canvas_part(
        &mut self,
        pos_size: Quad,
        canvas: &Framebuffer,
        uv_pos: Vec2,
        uv_size: Vec2,
        custom_shader: Option<CanvasShader>,
    ) {
        let uv_x1 = uv_pos.x();
        let uv_y1 = uv_pos.y();
        let uv_x2 = uv_pos.x() + uv_size.x();
        let uv_y2 = uv_pos.y() + uv_size.y();

        let p1 = self.affine_transform.apply(&pos_size.p1);
        let p2 = self.affine_transform.apply(&pos_size.p2);
        let p3 = self.affine_transform.apply(&pos_size.p3);
        let p4 = self.affine_transform.apply(&pos_size.p4);

        let texture = Some(BatchTexture {
            bind_group: canvas.bind_group().clone(),
            texture: canvas.color_texture().clone(),
        });

        // The y of the texture coordinates of a canvas goes up, unlike the one of an image: see `RenderTarget::flip_y`.
        if let Some(shader) = custom_shader {
            #[rustfmt::skip]
            let vertices: [f32; 4 * 4] = [
                // positions    // tex coords
                p4.x(), p4.y(), uv_x1, uv_y2, // top left
                p3.x(), p3.y(), uv_x2, uv_y2, // top right
                p2.x(), p2.y(), uv_x2, uv_y1, // bottom right
                p1.x(), p1.y(), uv_x1, uv_y1, // bottom left
            ];
            self.add_to_batch_by_trying_to_merge(
                &vertices,
                &INDICES_FOR_QUAD,
                BatchShader::Custom(shader.id),
                texture,
                shader.uniforms,
            );
        } else {
            #[rustfmt::skip]
            let vertices: [f32; 4 * 8] = [
                // positions    // tex coords  // tint
                p4.x(), p4.y(), uv_x1, uv_y2, 1.0, 1.0, 1.0, 1.0, // top left
                p3.x(), p3.y(), uv_x2, uv_y2, 1.0, 1.0, 1.0, 1.0, // top right
                p2.x(), p2.y(), uv_x2, uv_y1, 1.0, 1.0, 1.0, 1.0, // bottom right
                p1.x(), p1.y(), uv_x1, uv_y1, 1.0, 1.0, 1.0, 1.0, // bottom left
            ];
            self.add_to_batch_by_trying_to_merge(
                &vertices,
                &INDICES_FOR_QUAD,
                BatchShader::Texture,
                texture,
                None,
            );
        }
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
        let scale = font_size.abs() / font_resource.font_size;
        let mut vertices = Vec::<f32>::new();
        let mut indices = Vec::<u32>::new();
        let mut x_pos = 0.0;
        let mut y_pos = 0.0;
        let [r, g, b, a] = color;

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

                let p1 = self.affine_transform.apply(&Vec2::new(x0, y0));
                let p2 = self.affine_transform.apply(&Vec2::new(x1, y0));
                let p3 = self.affine_transform.apply(&Vec2::new(x1, y1));
                let p4 = self.affine_transform.apply(&Vec2::new(x0, y1));

                let base_index = (vertices.len() / 8) as u32; // Each vertex has 8 components

                #[rustfmt::skip]
                let s = &[
                    // positions    // tex coords  // color
                    p1.x(), p1.y(), s0, t1, r, g, b, a, // bottom left
                    p2.x(), p2.y(), s1, t1, r, g, b, a, // bottom right
                    p3.x(), p3.y(), s1, t0, r, g, b, a, // top right
                    p4.x(), p4.y(), s0, t0, r, g, b, a, // top left
                ];

                vertices.extend_from_slice(s);

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

        self.add_to_batch_by_trying_to_merge(
            &vertices,
            &indices,
            BatchShader::Font,
            Some(BatchTexture::from_texture(&font_resource.font_atlas)),
            None,
        );
    }

    pub fn flush(&mut self) {
        self.entries.clear();
    }

    pub fn clear(&self, color: [f32; 4]) {
        self.gfx.clear(color);
    }
}

/// Copies `data` to a buffer of the graphics card, which is replaced by a larger one if it is too small.
fn write_buffer(
    gfx: &Gfx,
    buffer: &mut Option<wgpu::Buffer>,
    usage: wgpu::BufferUsages,
    data: &[u8],
) -> wgpu::Buffer {
    let gpu = gfx.gpu();
    let needed = data.len() as wgpu::BufferAddress;
    let buffer = match buffer {
        Some(buffer) if buffer.size() >= needed => buffer,
        _ => buffer.insert(gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("batch"),
            size: needed.next_power_of_two().max(1 << 16),
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })),
    };
    gpu.queue.write_buffer(buffer, 0, data);
    buffer.clone()
}

const INDICES_FOR_QUAD: [u32; 6] = [
    0, 1, 2, // first triangle
    2, 3, 0, // second triangle
];

pub fn make_rect(x: f32, y: f32, width: f32, height: f32) -> Quad {
    let x_μ = f32::min(x, x + width);
    let x_ω = f32::max(x, x + width);
    let y_μ = f32::min(y, y + height);
    let y_ω = f32::max(y, y + height);

    Quad {
        p1: Vec2::new(x_μ, y_μ),
        p2: Vec2::new(x_ω, y_μ),
        p3: Vec2::new(x_ω, y_ω),
        p4: Vec2::new(x_μ, y_ω),
    }
}
