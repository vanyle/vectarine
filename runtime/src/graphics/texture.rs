use std::sync::Arc;

use vectarine_plugin_sdk::wgpu;

use crate::graphics::gpu::Gpu;

/// Represents a texture on the GPU
#[derive(Debug, Clone)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl Texture {
    /// Create a new RGBA texture
    pub fn new_rgba(
        gpu: &Gpu,
        data: Option<&[u8]>,
        width: u32,
        height: u32,
        filter: ImageAntialiasing,
    ) -> Arc<Self> {
        if let Some(data) = data {
            assert!(data.len() as u32 == width * height * 4);
        }
        let sampler = match filter {
            ImageAntialiasing::Nearest => &gpu.nearest_repeat_sampler,
            ImageAntialiasing::Linear | ImageAntialiasing::LinearWithMipmaps => {
                &gpu.linear_repeat_sampler
            }
        };
        Self::new(
            gpu,
            wgpu::TextureFormat::Rgba8Unorm,
            4,
            data,
            width,
            height,
            sampler,
        )
    }

    /// Create a new texture with 1 byte per pixel
    pub fn new_grayscale(gpu: &Gpu, data: &[u8], width: u32, height: u32) -> Arc<Self> {
        assert!(data.len() as u32 == width * height);
        Self::new(
            gpu,
            wgpu::TextureFormat::R8Unorm,
            1,
            Some(data),
            width,
            height,
            &gpu.linear_clamp_sampler,
        )
    }

    fn new(
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        bytes_per_pixel: u32,
        data: Option<&[u8]>,
        width: u32,
        height: u32,
        sampler: &wgpu::Sampler,
    ) -> Arc<Self> {
        let size = wgpu::Extent3d {
            // An empty texture is not allowed
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        if let Some(data) = data
            && width > 0
            && height > 0
        {
            gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * bytes_per_pixel),
                    rows_per_image: Some(height),
                },
                size,
            );
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = gpu.create_texture_bind_group(&view, sampler);

        Arc::new(Self {
            texture,
            view,
            bind_group,
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn raw(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// The texture and its sampler, ready to be given to a shader.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageAntialiasing {
    Nearest,
    Linear,
    LinearWithMipmaps,
}
