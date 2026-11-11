use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::Arc,
};

use vectarine_plugin_sdk::wgpu;

use crate::graphics::{
    framebuffer::Viewport,
    gpu::Gpu,
    pipeline::{
        BuiltinShader, MASK_STENCIL_REFERENCE, PipelineShaders, StencilMode, TargetKind,
        create_pipeline,
    },
};

/// Something that can be drawn on: a window or a canvas.
pub struct RenderTarget {
    /// Where the drawings go. With antialiasing, this is a multisampled texture and `resolve` is the final image.
    pub color: wgpu::TextureView,
    pub resolve: Option<wgpu::TextureView>,
    pub depth_stencil: wgpu::TextureView,
    /// The final image, to notice when it is drawn on itself.
    pub texture: wgpu::Texture,
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub width: u32,
    pub height: u32,
    /// Canvases are stored upside down, like OpenGL did: the first row of their texture is their bottom.
    /// This keeps the y axis of texture coordinates going up when a canvas is drawn or given to a shader.
    pub flip_y: bool,
}

struct ActiveTarget {
    target: RenderTarget,
    viewport: Viewport,
    pending_clear: Option<[f32; 4]>,
    pending_stencil_clear: bool,
}

struct State {
    /// The window at the bottom, then the canvases being painted. Drawings go to the last one.
    targets: Vec<ActiveTarget>,
    stencil_mode: StencilMode,
}

struct Inner {
    gpu: Arc<Gpu>,
    state: RefCell<State>,
    draw_call_counter: Cell<usize>,
    builtin_pipelines: RefCell<HashMap<(BuiltinShader, TargetKind), wgpu::RenderPipeline>>,
    color_layout: wgpu::PipelineLayout,
    texture_layout: wgpu::PipelineLayout,
}

/// What a render pass draws on.
pub struct PassInfo {
    pub target: TargetKind,
    pub texture: wgpu::Texture,
}

/// The state of the drawing: where drawings go, and how. Cloning it gives another handle to the same state.
#[derive(Clone)]
pub struct Gfx {
    inner: Rc<Inner>,
}

impl Gfx {
    pub fn new(gpu: Arc<Gpu>) -> Self {
        let layout = |label, bind_group_layouts: &[Option<&wgpu::BindGroupLayout>]| {
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts,
                    immediate_size: 0,
                })
        };
        Self {
            inner: Rc::new(Inner {
                color_layout: layout("color", &[]),
                texture_layout: layout("texture", &[Some(&gpu.texture_layout)]),
                gpu,
                state: RefCell::new(State {
                    targets: Vec::new(),
                    stencil_mode: StencilMode::Disabled,
                }),
                draw_call_counter: Cell::new(0),
                builtin_pipelines: RefCell::new(HashMap::new()),
            }),
        }
    }

    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.inner.gpu
    }

    /// Sets what receives the drawings made outside of a canvas: usually the window.
    /// Without a target, drawings are discarded.
    pub fn set_root_target(&self, target: Option<RenderTarget>) {
        while self.has_target() {
            self.pop_target();
        }
        if let Some(target) = target {
            self.push_target(target);
        }
    }

    pub fn has_target(&self) -> bool {
        !self.inner.state.borrow().targets.is_empty()
    }

    pub fn push_target(&self, target: RenderTarget) {
        let viewport = Viewport::from_size(target.width as i32, target.height as i32);
        self.inner.state.borrow_mut().targets.push(ActiveTarget {
            target,
            viewport,
            pending_clear: None,
            pending_stencil_clear: false,
        });
    }

    pub fn pop_target(&self) {
        self.apply_pending_clear();
        self.inner.state.borrow_mut().targets.pop();
    }

    /// Restricts the drawings to an area of the current target.
    pub fn set_viewport(&self, viewport: Viewport) {
        if let Some(active) = self.inner.state.borrow_mut().targets.last_mut() {
            active.viewport = viewport;
        }
    }

    /// True if the drawings sent to the current target need their y axis reversed.
    pub fn flip_y(&self) -> bool {
        let state = self.inner.state.borrow();
        state
            .targets
            .last()
            .is_some_and(|active| active.target.flip_y)
    }

    pub fn viewport(&self) -> Option<Viewport> {
        self.inner
            .state
            .borrow()
            .targets
            .last()
            .map(|active| active.viewport)
    }

    /// Fills the current target with a color. Like with OpenGL, this happens before the drawings waiting in a batch.
    pub fn clear(&self, color: [f32; 4]) {
        let mut state = self.inner.state.borrow_mut();
        // Nothing is visible while a mask is drawn
        if state.stencil_mode == StencilMode::WriteMask {
            return;
        }
        if let Some(active) = state.targets.last_mut() {
            active.pending_clear = Some(color);
        }
    }

    /// Clears the current target now if `clear` was called, instead of waiting for the next drawing.
    pub fn apply_pending_clear(&self) {
        let is_clear_pending = self
            .inner
            .state
            .borrow()
            .targets
            .last()
            .is_some_and(|active| active.pending_clear.is_some());
        if is_clear_pending {
            self.render(|_, _| {});
        }
    }

    pub fn stencil_mode(&self) -> StencilMode {
        self.inner.state.borrow().stencil_mode
    }

    pub fn set_stencil_mode(&self, stencil_mode: StencilMode) {
        self.inner.state.borrow_mut().stencil_mode = stencil_mode;
    }

    /// Empties the mask of the current target before the next drawing.
    pub fn clear_stencil(&self) {
        if let Some(active) = self.inner.state.borrow_mut().targets.last_mut() {
            active.pending_stencil_clear = true;
        }
    }

    /// Draws on the current target. Returns false without calling `draw` if there is nothing to draw on.
    pub fn render(&self, draw: impl FnOnce(&mut wgpu::RenderPass<'_>, &PassInfo)) -> bool {
        let gpu = &self.inner.gpu;
        let (color, resolve, depth_stencil, info, viewport, clear, clear_stencil) = {
            let mut state = self.inner.state.borrow_mut();
            let stencil_mode = state.stencil_mode;
            let Some(active) = state.targets.last_mut() else {
                return false;
            };
            (
                active.target.color.clone(),
                active.target.resolve.clone(),
                active.target.depth_stencil.clone(),
                PassInfo {
                    target: TargetKind {
                        format: active.target.format,
                        sample_count: active.target.sample_count,
                        stencil_mode,
                    },
                    texture: active.target.texture.clone(),
                },
                active.viewport,
                active.pending_clear.take(),
                std::mem::take(&mut active.pending_stencil_clear),
            )
        };

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color,
                    depth_slice: None,
                    resolve_target: resolve.as_ref(),
                    ops: wgpu::Operations {
                        load: match clear {
                            Some([r, g, b, a]) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: r as f64,
                                g: g as f64,
                                b: b as f64,
                                a: a as f64,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_stencil,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: if clear_stencil {
                            wgpu::LoadOp::Clear(0)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let max_size = gpu.device.limits().max_texture_dimension_2d as i32;
            let width = viewport.width.min(max_size);
            let height = viewport.height.min(max_size);
            if width > 0 && height > 0 {
                pass.set_viewport(
                    viewport.x as f32,
                    viewport.y as f32,
                    width as f32,
                    height as f32,
                    0.0,
                    1.0,
                );
                pass.set_stencil_reference(MASK_STENCIL_REFERENCE);
                draw(&mut pass, &info);
            }
        }
        gpu.queue.submit([encoder.finish()]);
        true
    }

    pub fn builtin_pipeline(
        &self,
        shader: BuiltinShader,
        target: TargetKind,
    ) -> wgpu::RenderPipeline {
        self.inner
            .builtin_pipelines
            .borrow_mut()
            .entry((shader, target))
            .or_insert_with(|| {
                let gpu = &self.inner.gpu;
                let (vertex_entry_point, fragment_entry_point) = shader.entry_points();
                create_pipeline(
                    gpu,
                    &PipelineShaders {
                        label: fragment_entry_point,
                        layout: match shader {
                            BuiltinShader::Color => &self.inner.color_layout,
                            BuiltinShader::Texture | BuiltinShader::Font => {
                                &self.inner.texture_layout
                            }
                        },
                        vertex_module: &gpu.builtin_shaders,
                        vertex_entry_point,
                        vertex_format: shader.vertex_format(),
                        fragment_module: &gpu.builtin_shaders,
                        fragment_entry_point,
                    },
                    target,
                )
            })
            .clone()
    }

    pub fn count_draw_call(&self) {
        self.inner
            .draw_call_counter
            .set(self.inner.draw_call_counter.get() + 1);
    }

    pub fn get_draw_call_counter(&self) -> usize {
        self.inner.draw_call_counter.get()
    }

    pub fn reset_draw_call_counter(&self) {
        self.inner.draw_call_counter.set(0);
    }
}
