use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use vectarine_plugin_sdk::sdl2::video::Window;
use vectarine_plugin_sdk::wgpu::{
    self,
    rwh::{DisplayHandle, HandleError, HasDisplayHandle, RawDisplayHandle},
};

use crate::{console, graphics::shadersources::BUILTIN_SHADERS};

pub const DEPTH_STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;
pub const CANVAS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// The connection to the graphics card, and what every drawing shares.
pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    /// Group 0 of the shaders that read a texture: the texture at binding 0, its sampler at binding 1.
    pub texture_layout: wgpu::BindGroupLayout,
    /// Group 1 of the custom shaders: their uniforms at binding 0.
    pub uniform_layout: wgpu::BindGroupLayout,
    pub builtin_shaders: wgpu::ShaderModule,

    pub nearest_repeat_sampler: wgpu::Sampler,
    pub linear_repeat_sampler: wgpu::Sampler,
    pub linear_clamp_sampler: wgpu::Sampler,

    id: u64,
}

#[derive(Debug)]
struct SdlDisplay(RawDisplayHandle);

// Safety: the handle is only an identifier of the display, wgpu uses it on the thread that owns the window.
unsafe impl Send for SdlDisplay {}
unsafe impl Sync for SdlDisplay {}

impl HasDisplayHandle for SdlDisplay {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // Safety: SDL keeps the display alive until it quits.
        Ok(unsafe { DisplayHandle::borrow_raw(self.0) })
    }
}

impl Gpu {
    /// Connects to the graphics card that can draw on `window`, and returns the surface of the window.
    ///
    /// # Safety
    /// The window must outlive the surface returned.
    pub unsafe fn for_window(
        window: &Window,
    ) -> Result<(Arc<Self>, wgpu::Surface<'static>), String> {
        let display = window
            .display_handle()
            .map_err(|e| format!("Failed to get the display of the window: {e}"))?
            .as_raw();
        // The OpenGL backend (the one used on the web) needs to know the display from the start.
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(SdlDisplay(display)))
                .with_env(),
        );
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(window) }
            .map_err(|e| format!("Failed to get the handle of the window: {e}"))?;
        let surface = unsafe { instance.create_surface_unsafe(target) }
            .map_err(|e| format!("Failed to create a drawing surface for the window: {e}"))?;
        let gpu = Self::new(instance, Some(&surface))?;
        Ok((gpu, surface))
    }

    /// Connects to a graphics card without any window, to draw on images only.
    pub fn headless() -> Result<Arc<Self>, String> {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle().with_env());
        Self::new(instance, None)
    }

    fn new(
        instance: wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Arc<Self>, String> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::from_env().unwrap_or_default(),
            force_fallback_adapter: false,
            compatible_surface,
            ..Default::default()
        }))
        .map_err(|e| format!("No graphics card can be used: {e}"))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("vectarine"),
            // The limits of WebGL 2, so that a game behaves the same on all platforms.
            required_limits:
                wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        }))
        .map_err(|e| format!("Failed to open the graphics card: {e}"))?;

        // Without this, wgpu panics on errors like a canvas that is too large.
        device.on_uncaptured_error(Arc::new(|error| {
            console::print_err(format!("Graphics error: {error}"));
        }));

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniforms"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let builtin_shaders = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("builtin shaders"),
            source: wgpu::ShaderSource::Wgsl(BUILTIN_SHADERS.into()),
        });

        let sampler = |filter: wgpu::FilterMode, address_mode: wgpu::AddressMode| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: address_mode,
                address_mode_v: address_mode,
                mag_filter: filter,
                min_filter: filter,
                ..Default::default()
            })
        };

        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Ok(Arc::new(Self {
            nearest_repeat_sampler: sampler(wgpu::FilterMode::Nearest, wgpu::AddressMode::Repeat),
            linear_repeat_sampler: sampler(wgpu::FilterMode::Linear, wgpu::AddressMode::Repeat),
            linear_clamp_sampler: sampler(wgpu::FilterMode::Linear, wgpu::AddressMode::ClampToEdge),
            texture_layout,
            uniform_layout,
            builtin_shaders,
            instance,
            adapter,
            device,
            queue,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }))
    }

    /// A number that is different for every `Gpu` created by this process.
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn create_texture_bind_group(
        &self,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    /// The number of samples per pixel to use when drawing on a window of this format (antialiasing).
    pub fn sample_count_for(&self, format: wgpu::TextureFormat) -> u32 {
        let flags = self.adapter.get_texture_format_features(format).flags;
        if flags.sample_count_supported(4) {
            4
        } else {
            1
        }
    }

    /// The color and depth+stencil textures used to draw on an image of this size.
    /// Without antialiasing (`sample_count` is 1), there is no color texture: drawings go to the image directly.
    pub fn create_attachments(
        &self,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> (Option<wgpu::TextureView>, wgpu::TextureView) {
        let create = |label, format| {
            self.device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: width.max(1),
                        height: height.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        let multisampled_color = (sample_count > 1).then(|| create("multisampled color", format));
        (
            multisampled_color,
            create("depth stencil", DEPTH_STENCIL_FORMAT),
        )
    }
}
