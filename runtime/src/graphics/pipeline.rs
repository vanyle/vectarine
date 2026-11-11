use vectarine_plugin_sdk::wgpu;

use crate::graphics::gpu::{DEPTH_STENCIL_FORMAT, Gpu};

/// The value written in the stencil buffer where a mask is drawn.
pub const MASK_STENCIL_REFERENCE: u32 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StencilMode {
    /// Drawings are not masked.
    Disabled,
    /// Drawings are invisible: they define the area of the mask.
    WriteMask,
    /// Drawings only appear inside the mask.
    TestMask,
}

/// What a pipeline depends on, other than its shaders.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TargetKind {
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub stencil_mode: StencilMode,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BuiltinShader {
    Color,
    Texture,
    Font,
}

impl BuiltinShader {
    pub fn vertex_format(self) -> VertexFormat {
        match self {
            BuiltinShader::Color => VertexFormat::PositionColor,
            BuiltinShader::Texture | BuiltinShader::Font => VertexFormat::PositionUvColor,
        }
    }
    pub(crate) fn entry_points(self) -> (&'static str, &'static str) {
        match self {
            BuiltinShader::Color => ("vs_color", "fs_color"),
            BuiltinShader::Texture => ("vs_texture", "fs_texture"),
            BuiltinShader::Font => ("vs_texture", "fs_font"),
        }
    }
}

/// How the vertices given to a shader are made. All the values are f32.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum VertexFormat {
    /// x, y, r, g, b, a
    PositionColor,
    /// x, y, u, v, r, g, b, a
    PositionUvColor,
    /// x, y, u, v
    PositionUv,
}

impl VertexFormat {
    pub const fn floats_per_vertex(self) -> usize {
        match self {
            VertexFormat::PositionColor => 6,
            VertexFormat::PositionUvColor => 8,
            VertexFormat::PositionUv => 4,
        }
    }

    fn attributes(self) -> &'static [wgpu::VertexAttribute] {
        const POSITION_COLOR: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
        const POSITION_UV_COLOR: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];
        const POSITION_UV: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
        match self {
            VertexFormat::PositionColor => &POSITION_COLOR,
            VertexFormat::PositionUvColor => &POSITION_UV_COLOR,
            VertexFormat::PositionUv => &POSITION_UV,
        }
    }
}

pub struct PipelineShaders<'a> {
    pub label: &'a str,
    pub layout: &'a wgpu::PipelineLayout,
    pub vertex_module: &'a wgpu::ShaderModule,
    pub vertex_entry_point: &'a str,
    pub vertex_format: VertexFormat,
    pub fragment_module: &'a wgpu::ShaderModule,
    pub fragment_entry_point: &'a str,
}

pub fn create_pipeline(
    gpu: &Gpu,
    shaders: &PipelineShaders,
    target: TargetKind,
) -> wgpu::RenderPipeline {
    let (stencil_face, stencil_write_mask, color_write_mask) = match target.stencil_mode {
        StencilMode::Disabled => (wgpu::StencilFaceState::IGNORE, 0, wgpu::ColorWrites::ALL),
        StencilMode::WriteMask => (
            wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Always,
                fail_op: wgpu::StencilOperation::Replace,
                depth_fail_op: wgpu::StencilOperation::Replace,
                pass_op: wgpu::StencilOperation::Replace,
            },
            0xFF,
            wgpu::ColorWrites::empty(),
        ),
        StencilMode::TestMask => (
            wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            },
            0,
            wgpu::ColorWrites::ALL,
        ),
    };
    // Alpha is blended like the colors, which is what OpenGL's glBlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA) did.
    let blend = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(shaders.label),
            layout: Some(shaders.layout),
            vertex: wgpu::VertexState {
                module: shaders.vertex_module,
                entry_point: Some(shaders.vertex_entry_point),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: (shaders.vertex_format.floats_per_vertex() * size_of::<f32>())
                        as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: shaders.vertex_format.attributes(),
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_STENCIL_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState {
                    front: stencil_face,
                    back: stencil_face,
                    read_mask: 0xFF,
                    write_mask: stencil_write_mask,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: target.sample_count,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: shaders.fragment_module,
                entry_point: Some(shaders.fragment_entry_point),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target.format,
                    blend: Some(wgpu::BlendState {
                        color: blend,
                        alpha: blend,
                    }),
                    write_mask: color_write_mask,
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
}
