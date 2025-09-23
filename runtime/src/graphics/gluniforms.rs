#[derive(Debug, Clone)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat3([[f32; 3]; 3]),
    Mat4([[f32; 4]; 4]),
    Int(i32),
    Bool(bool),
    Sampler2D(glow::NativeTexture), // texture ID
    SamplerCube(u32),
}

impl PartialEq for UniformValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (UniformValue::Float(a), UniformValue::Float(b)) => a == b,
            (UniformValue::Vec2(a), UniformValue::Vec2(b)) => a == b,
            (UniformValue::Vec3(a), UniformValue::Vec3(b)) => a == b,
            (UniformValue::Vec4(a), UniformValue::Vec4(b)) => a == b,
            (UniformValue::Mat3(a), UniformValue::Mat3(b)) => a == b,
            (UniformValue::Mat4(a), UniformValue::Mat4(b)) => a == b,
            (UniformValue::Int(a), UniformValue::Int(b)) => a == b,
            (UniformValue::Bool(a), UniformValue::Bool(b)) => a == b,
            // Textures are compared by reference, not value
            (UniformValue::Sampler2D(a), UniformValue::Sampler2D(b)) => a == b,
            (UniformValue::SamplerCube(a), UniformValue::SamplerCube(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Uniforms {
    pub data: Vec<(String, UniformValue)>,
}

impl Uniforms {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(&mut self, name: &str, value: UniformValue) {
        self.data.push((name.to_string(), value));
    }

    pub fn set(&mut self, name: &str, value: UniformValue) {
        if let Some((_, v)) = self.data.iter_mut().find(|(n, _)| n == name) {
            *v = value;
        } else {
            self.add(name, value);
        }
    }

    pub fn get(&self, name: &str) -> Option<&UniformValue> {
        self.data.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// Two uniforms are similar if they are the same when ignoring the order of fields.
    /// Meaning, they represent the same shader state. Textures inside uniforms are compared by reference, not value.
    pub fn similar(&self, other: &Uniforms) -> bool {
        if self.data.len() != other.data.len() {
            return false;
        }
        for (name, value) in &self.data {
            let Some(other_value) = other.get(name) else {
                return false;
            };
            if value != other_value {
                return false;
            }
        }
        true
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self::new()
    }
}
