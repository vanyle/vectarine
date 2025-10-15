use std::collections::HashSet;

use crate::graphics::gluniforms::UniformValue;

/// Internal representation for OpenGL types used in shaders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GLTypes {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
    Int,
    Bool,
    Sampler2D,
    SamplerCube,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsageHint {
    Position,
    Normal,
    TexCoord,
    Color,
    Custom,
}

impl GLTypes {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            GLTypes::Float => 4,
            GLTypes::Vec2 => 8,
            GLTypes::Vec3 => 12,
            GLTypes::Vec4 => 16,
            GLTypes::Mat3 => 36,
            GLTypes::Mat4 => 64,
            GLTypes::Int => 4,
            GLTypes::Bool => 1,
            GLTypes::Sampler2D => 0,
            GLTypes::SamplerCube => 0,
        }
    }

    pub fn to_gl_enum(&self) -> u32 {
        match self {
            GLTypes::Float => glow::FLOAT,
            GLTypes::Vec2 => glow::FLOAT_VEC2,
            GLTypes::Vec3 => glow::FLOAT_VEC3,
            GLTypes::Vec4 => glow::FLOAT_VEC4,
            GLTypes::Mat3 => glow::FLOAT_MAT3,
            GLTypes::Mat4 => glow::FLOAT_MAT4,
            GLTypes::Int => glow::INT,
            GLTypes::Bool => glow::BOOL,
            GLTypes::Sampler2D => glow::SAMPLER_2D,
            GLTypes::SamplerCube => glow::SAMPLER_CUBE,
        }
    }

    pub fn to_gl_subtype(&self) -> u32 {
        match self {
            GLTypes::Float => glow::FLOAT,
            GLTypes::Vec2 => glow::FLOAT,
            GLTypes::Vec3 => glow::FLOAT,
            GLTypes::Vec4 => glow::FLOAT,
            GLTypes::Mat3 => glow::FLOAT,
            GLTypes::Mat4 => glow::FLOAT,
            GLTypes::Int => glow::INT,
            GLTypes::Bool => glow::BOOL,
            GLTypes::Sampler2D => glow::INT,
            GLTypes::SamplerCube => glow::INT,
        }
    }

    pub fn component_count(&self) -> usize {
        match self {
            GLTypes::Float => 1,
            GLTypes::Vec2 => 2,
            GLTypes::Vec3 => 3,
            GLTypes::Vec4 => 4,
            GLTypes::Mat3 => 9,
            GLTypes::Mat4 => 16,
            GLTypes::Int => 1,
            GLTypes::Bool => 1,
            GLTypes::Sampler2D => 0,
            GLTypes::SamplerCube => 0,
        }
    }

    pub fn matches_value(&self, value: &UniformValue) -> bool {
        matches!(
            (self, value),
            (GLTypes::Float, UniformValue::Float(_))
                | (GLTypes::Vec2, UniformValue::Vec2(_))
                | (GLTypes::Vec3, UniformValue::Vec3(_))
                | (GLTypes::Vec4, UniformValue::Vec4(_))
                | (GLTypes::Mat3, UniformValue::Mat3(_))
                | (GLTypes::Mat4, UniformValue::Mat4(_))
                | (GLTypes::Int, UniformValue::Int(_))
                | (GLTypes::Bool, UniformValue::Bool(_))
                | (GLTypes::Sampler2D, UniformValue::Sampler2D(_))
                | (GLTypes::SamplerCube, UniformValue::SamplerCube(_))
        )
    }
}

impl std::fmt::Display for GLTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GLTypes::Float => "float",
            GLTypes::Vec2 => "vec2",
            GLTypes::Vec3 => "vec3",
            GLTypes::Vec4 => "vec4",
            GLTypes::Mat3 => "mat3",
            GLTypes::Mat4 => "mat4",
            GLTypes::Int => "int",
            GLTypes::Bool => "bool",
            GLTypes::Sampler2D => "sampler2D",
            GLTypes::SamplerCube => "samplerCube",
        };
        write!(f, "{s}")
    }
}

/// Represents how a piece of data is supposed to be understood by the GPU.
/// This is akin to a type, but it exists at runtime for introspection
///
/// In OpenGL, all data is stored in a buffer containing bytes.
/// This array of bytes is interpreted as SomeType[] where SomeType is a struct with various fields.
/// DataLayout is a runtime representation of SomeType.
/// GpuVertexData are instances of that type.
#[derive(Debug, Clone)]
pub struct DataLayout {
    pub fields: Vec<(String, GLTypes, Option<UsageHint>)>,
}

impl DataLayout {
    pub fn new() -> Self {
        Self { fields: vec![] }
    }

    pub fn add_field(
        &mut self,
        name: &str,
        gl_type: GLTypes,
        usage: Option<UsageHint>,
    ) -> &mut Self {
        self.fields.push((name.to_string(), gl_type, usage));
        self
    }

    /// Returns the size in bytes of one row of the layout
    pub fn stride(&self) -> usize {
        self.fields.iter().map(|(_, t, _)| t.size_in_bytes()).sum()
    }

    /// Checks if a DataLayout is valid for an array buffer.
    pub fn is_valid_for_vertex(&self) -> bool {
        self.fields
            .iter()
            .all(|(_, t, _)| !matches!(t, GLTypes::Sampler2D | GLTypes::SamplerCube))
    }

    /// Checks if a vertex data containing the provided vertices and indices
    /// would by valid for this layout
    /// We assume that vertices[0] is the contains the vertex_offset-th vertex.
    /// This is useful when merging VertexData together.
    /// We also assume that indices can only reference the vertices, and never any previous ones.
    pub fn is_sound(
        &self,
        vertices: &[u8],
        indices: &[u32],
        idx_of_first_vertex: usize,
    ) -> Option<String> {
        let stride = self.stride();
        // 0 data per row means the buffer needs to be empty for this to be valid.
        if stride == 0 {
            if vertices.is_empty() && indices.is_empty() {
                return None;
            } else {
                return Some("Layout has no data, but buffer is not empty".to_string());
            }
        }
        // Row is incomplete
        if !vertices.len().is_multiple_of(stride) {
            return Some(format!(
                "A row is incomplete, the row is made of {stride} bytes but the vertex buffer has {} bytes",
                vertices.len()
            ));
        }
        // We assume that triangles are drawn
        if !indices.len().is_multiple_of(3) {
            return Some(
                "Index buffer is not a multiple of 3, but we are drawing triangles".to_string(),
            );
        }
        let vertex_count = vertices.len() / stride;
        for i in indices.iter() {
            let idx = *i as usize;
            if idx >= vertex_count + idx_of_first_vertex && idx < idx_of_first_vertex {
                return Some(format!(
                    "Index buffer is not valid, {} is outside {}..<{}, the bounds of the vertex data",
                    idx,
                    idx_of_first_vertex,
                    vertex_count + idx_of_first_vertex
                ));
            }
        }
        None
    }

    /// Checks that every vertex provided in the buffer is used by at least one index.
    /// If the data is not found, we also return false
    /// This method requires us to interate over all vertices to check that they are used, so it can be slow for large buffers.
    pub fn is_not_wasteful(
        &self,
        vertices: &[u8],
        indices: &[u32],
        idx_of_first_vertex: usize,
    ) -> bool {
        if self
            .is_sound(vertices, indices, idx_of_first_vertex)
            .is_some()
        {
            return false;
        }
        let stride = self.stride();
        let vertex_count = vertices.len() / stride;
        let indices_used = HashSet::<u32>::from_iter(indices.iter().cloned());
        (0..vertex_count).all(|i| indices_used.contains(&((i + idx_of_first_vertex) as u32)))
    }
}

impl std::default::Default for DataLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DataLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, gl_type, usage) in &self.fields {
            if let Some(usage) = usage {
                writeln!(f, "{name}: {gl_type} ({usage:?})")?;
            } else {
                writeln!(f, "{name}: {gl_type}")?;
            }
        }
        Ok(())
    }
}
