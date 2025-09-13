use std::sync::Arc;

use crate::graphics::gltexture::Texture;

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
    Sampler2D(Arc<Texture>), // texture ID
    SamplerCube(u32),
}

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
}

impl Default for Uniforms {
    fn default() -> Self {
        Self::new()
    }
}
