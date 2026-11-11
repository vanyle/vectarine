use vectarine_plugin_sdk::wgpu;

use crate::graphics::{
    gfx::{Gfx, RenderTarget},
    gpu::{CANVAS_FORMAT, Gpu},
};

/// The textures needed to draw on an image, other than the image itself.
struct Attachments {
    multisampled_color: Option<wgpu::TextureView>,
    depth_stencil: wgpu::TextureView,
}

impl Attachments {
    fn new(
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> Self {
        let (multisampled_color, depth_stencil) =
            gpu.create_attachments(format, width, height, sample_count);
        Self {
            multisampled_color,
            depth_stencil,
        }
    }

    fn target(&self, texture: &wgpu::Texture, sample_count: u32) -> RenderTarget {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let (color, resolve) = match &self.multisampled_color {
            Some(multisampled_color) => (multisampled_color.clone(), Some(view)),
            None => (view, None),
        };
        RenderTarget {
            color,
            resolve,
            depth_stencil: self.depth_stencil.clone(),
            texture: texture.clone(),
            format: texture.format(),
            sample_count,
            width: texture.width(),
            height: texture.height(),
            flip_y: false,
        }
    }
}

/// What is shown in a window.
pub struct WindowSurface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    sample_count: u32,
    attachments: Attachments,
    frame: Option<wgpu::SurfaceTexture>,
}

impl WindowSurface {
    /// `surface` needs to be a surface of `gpu`: see `Gpu::for_window` and `WindowSurface::for_window`.
    pub fn new(gpu: &Gpu, surface: wgpu::Surface<'static>, width: u32, height: u32) -> Self {
        let capabilities = surface.get_capabilities(&gpu.adapter);
        // Colors are blended as they are stored (not linearly), like with OpenGL: no sRGB format.
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| !format.is_srgb())
            .or(capabilities.formats.first().copied())
            .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);
        let alpha_mode = if capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Opaque)
        {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            wgpu::CompositeAlphaMode::Auto
        };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: Default::default(),
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&gpu.device, &config);
        let sample_count = gpu.sample_count_for(format);

        Self {
            attachments: Attachments::new(gpu, format, config.width, config.height, sample_count),
            surface,
            config,
            sample_count,
            frame: None,
        }
    }

    /// Creates the surface of another window than the one given to `Gpu::for_window`.
    ///
    /// # Safety
    /// The window must outlive the surface returned.
    pub unsafe fn for_window(
        gpu: &Gpu,
        window: &vectarine_plugin_sdk::sdl2::video::Window,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(window) }
            .map_err(|e| format!("Failed to get the handle of the window: {e}"))?;
        let surface = unsafe { gpu.instance.create_surface_unsafe(target) }
            .map_err(|e| format!("Failed to create a drawing surface for the window: {e}"))?;
        Ok(Self::new(gpu, surface, width, height))
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    fn resize(&mut self, gpu: &Gpu, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&gpu.device, &self.config);
        self.attachments =
            Attachments::new(gpu, self.config.format, width, height, self.sample_count);
    }

    /// Makes the window the target of the drawings, until `end_frame`. `width` and `height` are the size of the
    /// window in pixels. Returns false if the window cannot be drawn on right now (when it is hidden for example).
    pub fn begin_frame(&mut self, gfx: &Gfx, width: u32, height: u32) -> bool {
        let gpu = gfx.gpu();
        if width == 0 || height == 0 {
            return false;
        }
        if width != self.config.width || height != self.config.height {
            self.resize(gpu, width, height);
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(_)
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&gpu.device, &self.config);
                match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                    _ => return false,
                }
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return false,
        };
        gfx.set_root_target(Some(
            self.attachments.target(&frame.texture, self.sample_count),
        ));
        self.frame = Some(frame);
        true
    }

    /// The image shown at the end of the frame, to draw on it without going through `Gfx`.
    pub fn frame_view(&self) -> Option<wgpu::TextureView> {
        self.frame.as_ref().map(|frame| {
            frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default())
        })
    }

    /// Shows what was drawn since `begin_frame`.
    pub fn end_frame(&mut self, gfx: &Gfx) {
        gfx.set_root_target(None);
        if let Some(frame) = self.frame.take() {
            gfx.gpu().queue.present(frame);
        }
    }
}

/// An image used in place of a window, when there is no screen.
pub struct OffscreenSurface {
    texture: wgpu::Texture,
    sample_count: u32,
    attachments: Attachments,
}

impl OffscreenSurface {
    pub fn new(gpu: &Gpu, width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen surface"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: CANVAS_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let sample_count = gpu.sample_count_for(CANVAS_FORMAT);
        Self {
            attachments: Attachments::new(gpu, CANVAS_FORMAT, width, height, sample_count),
            texture,
            sample_count,
        }
    }

    /// Makes the image the target of the drawings, until `end_frame`.
    pub fn begin_frame(&mut self, gfx: &Gfx, width: u32, height: u32) {
        if width.max(1) != self.texture.width() || height.max(1) != self.texture.height() {
            *self = Self::new(gfx.gpu(), width, height);
        }
        gfx.set_root_target(Some(
            self.attachments.target(&self.texture, self.sample_count),
        ));
    }

    pub fn end_frame(&self, gfx: &Gfx) {
        gfx.set_root_target(None);
    }

    /// Returns the RGBA pixels of the image, from its top row to its bottom row, with its width and height.
    pub fn read_pixels(&self, gpu: &Gpu) -> Result<(Vec<u8>, u32, u32), String> {
        let width = self.texture.width();
        let height = self.texture.height();
        let bytes_per_row = width * 4;
        let padded_bytes_per_row =
            bytes_per_row.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screenshot"),
            size: (padded_bytes_per_row * height) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            self.texture.size(),
        );
        gpu.queue.submit([encoder.finish()]);

        let (sender, receiver) = std::sync::mpsc::channel();
        buffer.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = sender.send(result);
        });
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("Failed to wait for the graphics card: {e}"))?;
        receiver
            .recv()
            .map_err(|e| format!("Failed to read the image: {e}"))?
            .map_err(|e| format!("Failed to read the image: {e}"))?;

        let data = buffer
            .get_mapped_range(..)
            .map_err(|e| format!("Failed to read the image: {e}"))?;
        let pixels = data
            .chunks_exact(padded_bytes_per_row as usize)
            .flat_map(|row| &row[..bytes_per_row as usize])
            .copied()
            .collect();
        Ok((pixels, width, height))
    }
}
