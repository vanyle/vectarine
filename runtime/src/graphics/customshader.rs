use std::{cell::RefCell, collections::HashMap};

use vectarine_plugin_sdk::wgpu::{self, naga};

use crate::graphics::{
    gpu::{CANVAS_FORMAT, Gpu},
    pipeline::{PipelineShaders, StencilMode, TargetKind, VertexFormat, create_pipeline},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniformKind {
    Float,
    Int,
    Uint,
    Vec2,
    Vec3,
    Vec4,
    /// A type that cannot be set from Lua, like a matrix.
    Unsupported,
}

impl UniformKind {
    fn name(self) -> &'static str {
        match self {
            UniformKind::Float => "f32",
            UniformKind::Int => "i32",
            UniformKind::Uint => "u32",
            UniformKind::Vec2 => "vec2f",
            UniformKind::Vec3 => "vec3f",
            UniformKind::Vec4 => "vec4f",
            UniformKind::Unsupported => "unsupported type",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayLayout {
    pub len: usize,
    /// Bytes between the start of an element and the start of the next.
    pub stride: usize,
}

/// A member of the uniform struct of a shader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniformField {
    pub name: String,
    pub offset: usize,
    pub kind: UniformKind,
    pub array: Option<ArrayLayout>,
}

impl UniformField {
    fn byte_len(&self) -> usize {
        let element_len = match self.kind {
            UniformKind::Float | UniformKind::Int | UniformKind::Uint => 4,
            UniformKind::Vec2 => 8,
            UniformKind::Vec3 => 12,
            UniformKind::Vec4 => 16,
            UniformKind::Unsupported => 0,
        };
        match &self.array {
            Some(array) if element_len > 0 => (array.len - 1) * array.stride + element_len,
            _ => element_len,
        }
    }
}

impl std::fmt::Display for UniformField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.array {
            Some(array) => write!(
                f,
                "{}: array<{}, {}>",
                self.name,
                self.kind.name(),
                array.len
            ),
            None => write!(f, "{}: {}", self.name, self.kind.name()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec4([f32; 4]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UniformError {
    NotFound,
    WrongType { expected: String },
    OutOfBounds { len: usize },
}

/// The name of the uniform filled by the engine with the seconds since the start of the game.
const TIME_UNIFORM: &str = "iTime";

#[derive(Clone, Copy)]
struct UniformLocation {
    offset: usize,
    field: usize,
}

/// A fragment shader written by the maker of a game, applied to a canvas when it is drawn.
pub struct CustomShader {
    module: wgpu::ShaderModule,
    entry_point: String,
    layout: wgpu::PipelineLayout,
    pipelines: RefCell<HashMap<TargetKind, wgpu::RenderPipeline>>,

    uniform_fields: Vec<UniformField>,
    /// Where the uniforms already set are, by the name given to `set_uniform` ("lights[2]" for example).
    uniform_locations: RefCell<HashMap<String, UniformLocation>>,
    /// Offset of the uniform that receives the time, if the shader has one.
    time_offset: Option<usize>,
    /// The values of the uniforms, as the shader reads them. Empty if the shader has no uniform.
    uniform_data: RefCell<Vec<u8>>,
    /// A drawing needs its own buffer to have its own values: buffers are reused from one frame to the next.
    uniform_slots: RefCell<Vec<(wgpu::Buffer, wgpu::BindGroup)>>,
}

impl CustomShader {
    pub fn from_source(gpu: &Gpu, source: &str) -> Result<Self, String> {
        let mut module =
            naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
        let validate = |module: &naga::Module| {
            naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::empty(),
            )
            .validate(module)
            .map_err(|e| e.emit_to_string(source))
        };
        validate(&module)?;

        let entry_point = module
            .entry_points
            .iter()
            .find(|entry_point| entry_point.stage == naga::ShaderStage::Fragment)
            .map(|entry_point| entry_point.name.clone())
            .ok_or("The shader has no fragment function. Add `@fragment` before the function that computes the color of a pixel.")?;

        let (uniform_fields, uniform_size) = read_uniforms(&mut module)?;
        let module_info = validate(&module)?;
        // In a browser, wgpu does this check itself when it compiles the shader.
        #[cfg(not(target_os = "emscripten"))]
        check_web_support(&module, &module_info, &entry_point)?;
        #[cfg(target_os = "emscripten")]
        let _ = module_info;

        let error_scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader_module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
            });
        let bind_group_layouts = [Some(&gpu.texture_layout), Some(&gpu.uniform_layout)];
        let used_bind_group_count = if uniform_size > 0 { 2 } else { 1 };
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bind_group_layouts[..used_bind_group_count],
                immediate_size: 0,
            });
        let shader = Self {
            module: shader_module,
            entry_point,
            layout,
            pipelines: RefCell::new(HashMap::new()),
            time_offset: uniform_fields
                .iter()
                .find(|field| {
                    field.name == TIME_UNIFORM
                        && field.kind == UniformKind::Float
                        && field.array.is_none()
                })
                .map(|field| field.offset),
            uniform_locations: RefCell::new(HashMap::new()),
            uniform_fields,
            uniform_data: RefCell::new(vec![0; uniform_size]),
            uniform_slots: RefCell::new(Vec::new()),
        };
        // A pipeline is made now to find the mistakes that only show when the shader is put to use.
        shader.pipeline(
            gpu,
            TargetKind {
                format: CANVAS_FORMAT,
                sample_count: 1,
                stencil_mode: StencilMode::Disabled,
            },
        );
        if let Some(error) = pollster::block_on(error_scope.pop()) {
            return Err(format!(
                "{error}\n\nA shader can use `@location(0) uv: vec2f` as input, has to return a `@location(0) vec4f`, and can declare:\n\
                @group(0) @binding(0) var tex: texture_2d<f32>;\n\
                @group(0) @binding(1) var tex_sampler: sampler;\n\
                @group(1) @binding(0) var<uniform> uniforms: YourStruct;"
            ));
        }
        Ok(shader)
    }

    pub fn pipeline(&self, gpu: &Gpu, target: TargetKind) -> wgpu::RenderPipeline {
        self.pipelines
            .borrow_mut()
            .entry(target)
            .or_insert_with(|| {
                create_pipeline(
                    gpu,
                    &PipelineShaders {
                        label: "custom shader",
                        layout: &self.layout,
                        vertex_module: &gpu.builtin_shaders,
                        vertex_entry_point: "vs_custom",
                        vertex_format: VertexFormat::PositionUv,
                        fragment_module: &self.module,
                        fragment_entry_point: &self.entry_point,
                    },
                    target,
                )
            })
            .clone()
    }

    pub fn uniform_fields(&self) -> &[UniformField] {
        &self.uniform_fields
    }

    /// Sets the value of a uniform. `name` is the name of a member of the uniform struct, or `member[index]` for an
    /// element of an array.
    pub fn set_uniform(&self, name: &str, value: UniformValue) -> Result<(), UniformError> {
        let known_location = self.uniform_locations.borrow().get(name).copied();
        let location = match known_location {
            Some(location) => location,
            None => {
                let location = self.find_uniform(name)?;
                self.uniform_locations
                    .borrow_mut()
                    .insert(name.to_string(), location);
                location
            }
        };
        let UniformLocation { offset, field } = location;
        let field = &self.uniform_fields[field];

        let mut data = self.uniform_data.borrow_mut();
        let mut write = |bytes: &[u8]| data[offset..offset + bytes.len()].copy_from_slice(bytes);
        match (field.kind, value) {
            (UniformKind::Float, UniformValue::Float(v)) => write(&v.to_ne_bytes()),
            (UniformKind::Int, UniformValue::Float(v)) => write(&(v as i32).to_ne_bytes()),
            (UniformKind::Uint, UniformValue::Float(v)) => write(&(v as u32).to_ne_bytes()),
            (UniformKind::Vec2, UniformValue::Vec2(v)) => write(bytemuck::bytes_of(&v)),
            (UniformKind::Vec3, UniformValue::Vec4(v)) => write(bytemuck::cast_slice(&v[..3])),
            (UniformKind::Vec4, UniformValue::Vec4(v)) => write(bytemuck::bytes_of(&v)),
            _ => {
                return Err(UniformError::WrongType {
                    expected: field.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Finds where a uniform is from its name: `member`, or `member[index]` for an element of an array.
    fn find_uniform(&self, name: &str) -> Result<UniformLocation, UniformError> {
        let (name, index) = match name.strip_suffix(']').and_then(|name| name.split_once('[')) {
            Some((name, index)) => (name, index.trim().parse::<usize>().ok()),
            None => (name, None),
        };
        let (field_index, field) = self
            .uniform_fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == name)
            .ok_or(UniformError::NotFound)?;
        let offset = match (&field.array, index) {
            (None, None) => field.offset,
            (Some(array), Some(index)) if index < array.len => field.offset + index * array.stride,
            (Some(array), Some(_)) => return Err(UniformError::OutOfBounds { len: array.len }),
            _ => {
                return Err(UniformError::WrongType {
                    expected: field.to_string(),
                });
            }
        };
        Ok(UniformLocation {
            offset,
            field: field_index,
        })
    }

    /// The current values of the uniforms, to give to a drawing. None if the shader has no uniform.
    pub fn uniform_snapshot(&self, time_in_seconds: f32) -> Option<Vec<u8>> {
        let mut data = self.uniform_data.borrow().clone();
        if data.is_empty() {
            return None;
        }
        if let Some(offset) = self.time_offset {
            data[offset..offset + 4].copy_from_slice(&time_in_seconds.to_ne_bytes());
        }
        Some(data)
    }

    /// Keeps the values of the uniforms of the previous version of this shader, for the ones that did not change.
    pub fn copy_uniforms_from(&self, previous: &CustomShader) {
        let previous_data = previous.uniform_data.borrow();
        let mut data = self.uniform_data.borrow_mut();
        for field in &self.uniform_fields {
            let same_field = previous.uniform_fields.iter().find(|previous_field| {
                previous_field.name == field.name
                    && previous_field.kind == field.kind
                    && previous_field.array == field.array
            });
            if let Some(previous_field) = same_field {
                let len = field.byte_len();
                data[field.offset..field.offset + len].copy_from_slice(
                    &previous_data[previous_field.offset..previous_field.offset + len],
                );
            }
        }
    }

    /// The uniforms of a drawing, ready to be given to the shader. `slot` needs to be different for every drawing
    /// sent to the graphics card at the same time.
    pub fn uniform_bind_group(
        &self,
        gpu: &Gpu,
        slot: usize,
        data: &[u8],
    ) -> Option<wgpu::BindGroup> {
        let size = self.uniform_data.borrow().len();
        if size == 0 || data.len() != size {
            return None;
        }
        let mut slots = self.uniform_slots.borrow_mut();
        while slots.len() <= slot {
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("uniforms"),
                size: size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("uniforms"),
                layout: &gpu.uniform_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            slots.push((buffer, bind_group));
        }
        let (buffer, bind_group) = &slots[slot];
        gpu.queue.write_buffer(buffer, 0, data);
        Some(bind_group.clone())
    }
}

/// Finds the uniforms of a shader: the members of the struct at group 1, binding 0.
/// Returns them with the size in bytes of the struct (0 without uniforms).
fn read_uniforms(module: &mut naga::Module) -> Result<(Vec<UniformField>, usize), String> {
    let mut uniforms = module
        .global_variables
        .iter()
        .filter(|(_, variable)| variable.space == naga::AddressSpace::Uniform);
    let Some((_, variable)) = uniforms.next() else {
        return Ok((Vec::new(), 0));
    };
    let name = variable.name.as_deref().unwrap_or("?");
    if uniforms.next().is_some() {
        return Err(
            "A shader can only have one `var<uniform>`. Put all your uniforms in one struct."
                .to_string(),
        );
    }
    let binding = variable.binding.as_ref().map(|b| (b.group, b.binding));
    if binding != Some((1, 0)) {
        return Err(format!(
            "The uniforms need to be declared with `@group(1) @binding(0) var<uniform> {name}: ...`"
        ));
    }
    let naga::TypeInner::Struct { members, span } = &module.types[variable.ty].inner else {
        return Err(format!(
            "The type of `{name}` needs to be a struct, whose members are the uniforms."
        ));
    };

    let scalar_kind = |scalar: naga::Scalar| match (scalar.kind, scalar.width) {
        (naga::ScalarKind::Float, 4) => UniformKind::Float,
        (naga::ScalarKind::Sint, 4) => UniformKind::Int,
        (naga::ScalarKind::Uint, 4) => UniformKind::Uint,
        _ => UniformKind::Unsupported,
    };
    let kind_of = |inner: &naga::TypeInner| match inner {
        naga::TypeInner::Scalar(scalar) => scalar_kind(*scalar),
        naga::TypeInner::Vector { size, scalar } if scalar_kind(*scalar) == UniformKind::Float => {
            match size {
                naga::VectorSize::Bi => UniformKind::Vec2,
                naga::VectorSize::Tri => UniformKind::Vec3,
                naga::VectorSize::Quad => UniformKind::Vec4,
            }
        }
        _ => UniformKind::Unsupported,
    };

    let fields = members
        .iter()
        .map(|member| {
            let (kind, array) = match &module.types[member.ty].inner {
                naga::TypeInner::Array {
                    base,
                    size: naga::ArraySize::Constant(len),
                    stride,
                } => (
                    kind_of(&module.types[*base].inner),
                    Some(ArrayLayout {
                        len: len.get() as usize,
                        stride: *stride as usize,
                    }),
                ),
                inner => (kind_of(inner), None),
            };
            UniformField {
                name: member.name.clone().unwrap_or_default(),
                offset: member.offset as usize,
                kind,
                array,
            }
        })
        .collect();

    // WebGL only accepts uniform structs whose size is a multiple of 16 bytes: the end of the struct is padded, so
    // that the makers of games do not have to think about it.
    let padded_span = span.next_multiple_of(16);
    if padded_span != *span {
        let struct_type = variable.ty;
        let mut padded_type = module.types[struct_type].clone();
        if let naga::TypeInner::Struct { span, .. } = &mut padded_type.inner {
            *span = padded_span;
        }
        module.types.replace(struct_type, padded_type);
    }
    Ok((fields, padded_span as usize))
}

/// Checks that the shader can be translated for WebGL 2, which is what draws the game in a browser.
#[cfg(not(target_os = "emscripten"))]
fn check_web_support(
    module: &naga::Module,
    module_info: &naga::valid::ModuleInfo,
    entry_point: &str,
) -> Result<(), String> {
    use naga::back::glsl;

    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        ..Default::default()
    };
    let pipeline_options = glsl::PipelineOptions {
        shader_stage: naga::ShaderStage::Fragment,
        entry_point: entry_point.to_string(),
        multiview: None,
    };
    let mut output = String::new();
    glsl::Writer::new(
        &mut output,
        module,
        module_info,
        &options,
        &pipeline_options,
        naga::proc::BoundsCheckPolicies::default(),
    )
    .and_then(|mut writer| writer.write())
    .map(|_| ())
    .map_err(|error| {
        format!("This shader works on this computer, but would not work in a web export: {error}")
    })
}
