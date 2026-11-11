use vectarine_plugin_sdk::wgpu;

use crate::graphics::{
    gfx::{Gfx, RenderTarget},
    gpu::{CANVAS_FORMAT, Gpu},
    texture::ImageAntialiasing,
};

pub struct Framebuffer {
    color: wgpu::Texture,
    color_view: wgpu::TextureView,
    depth_stencil_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl Framebuffer {
    pub fn new_rgba(gpu: &Gpu, width: u32, height: u32, filter: ImageAntialiasing) -> Self {
        let color = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("canvas"),
            size: wgpu::Extent3d {
                // An empty texture is not allowed
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: CANVAS_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let color_view = color.create_view(&wgpu::TextureViewDescriptor::default());
        let (_, depth_stencil_view) = gpu.create_attachments(CANVAS_FORMAT, width, height, 1);
        let sampler = match filter {
            ImageAntialiasing::Nearest => &gpu.nearest_repeat_sampler,
            ImageAntialiasing::Linear | ImageAntialiasing::LinearWithMipmaps => {
                &gpu.linear_repeat_sampler
            }
        };
        let bind_group = gpu.create_texture_bind_group(&color_view, sampler);

        Self {
            color,
            color_view,
            depth_stencil_view,
            bind_group,
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Execute the closure with the framebuffer as the target of the drawings.
    /// The viewport is adjusted to match the framebuffer size during the execution of the closure.
    pub fn using(&self, gfx: &Gfx, f: impl FnOnce()) {
        gfx.push_target(RenderTarget {
            color: self.color_view.clone(),
            resolve: None,
            depth_stencil: self.depth_stencil_view.clone(),
            texture: self.color.clone(),
            format: CANVAS_FORMAT,
            sample_count: 1,
            width: self.color.width(),
            height: self.color.height(),
            flip_y: true,
        });
        f();
        gfx.pop_target();
    }

    pub fn color_texture(&self) -> &wgpu::Texture {
        &self.color
    }

    /// The color texture and its sampler, ready to be given to a shader.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/// An area of a drawing target, in pixels. (0, 0) is the top left of the target and y goes down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Viewport {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    pub fn from_size(width: i32, height: i32) -> Self {
        Viewport {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}
