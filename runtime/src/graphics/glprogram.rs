use std::sync::Arc;

use glow::HasContext;

use crate::graphics::{
    gltypes::DataLayout,
    gluniforms::{UniformValue, Uniforms},
};

pub struct GLProgram {
    pub vert_src: String,
    pub frag_src: String,
    program: glow::NativeProgram,

    pub vertex_layout: DataLayout,
    pub uniform_layout: DataLayout,
    gl: Arc<glow::Context>,
}

impl GLProgram {
    pub fn use_program(&self) {
        unsafe {
            let gl = self.gl.as_ref();
            gl.use_program(Some(self.program));
        }
    }
    pub fn stop_using(&self) {
        unsafe {
            let gl = self.gl.as_ref();
            gl.use_program(None);
        }
    }

    pub fn from_source(
        gl: &Arc<glow::Context>,
        vert_src: &str,
        frag_src: &str,
    ) -> Result<Self, String> {
        let program = unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let shader_version = "#version 300 es";

            let shaders = [
                (glow::VERTEX_SHADER, vert_src),
                (glow::FRAGMENT_SHADER, frag_src),
            ];

            let mut shader_ids = Vec::with_capacity(shaders.len());

            for (shader_type, shader_source) in shaders.iter() {
                let shader = gl.create_shader(*shader_type)?;
                gl.shader_source(shader, &format!("{shader_version}\n{shader_source}"));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    return Err(format!(
                        "Failed to compile shader: {}",
                        gl.get_shader_info_log(shader)
                    ));
                }
                gl.attach_shader(program, shader);
                shader_ids.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(format!(
                    "Failed to link program: {}",
                    gl.get_program_info_log(program)
                ));
            }

            for shader in shader_ids {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            program
        };

        Ok(Self {
            vert_src: vert_src.to_string(),
            frag_src: frag_src.to_string(),
            program,
            vertex_layout: DataLayout::new(),
            uniform_layout: DataLayout::new(),
            gl: gl.clone(),
        })
    }

    /// Assumes that the program is already in use.
    pub fn set_uniforms(&self, uniforms: &Uniforms) {
        let gl = self.gl.as_ref();
        for (uniform_name, uniform_value) in &uniforms.data {
            unsafe {
                let location = gl
                    .get_uniform_location(self.program, uniform_name.as_str())
                    .unwrap_or_else(|| {
                        panic!("The uniform {uniform_name} should exist in the shader")
                    });

                match uniform_value {
                    UniformValue::Float(v) => {
                        gl.uniform_1_f32(Some(&location), *v);
                    }
                    UniformValue::Vec2(v) => {
                        gl.uniform_2_f32(Some(&location), v[0], v[1]);
                    }
                    UniformValue::Vec3(v) => {
                        gl.uniform_3_f32(Some(&location), v[0], v[1], v[2]);
                    }
                    UniformValue::Vec4(v) => {
                        gl.uniform_4_f32(Some(&location), v[0], v[1], v[2], v[3]);
                    }
                    UniformValue::Mat3(v) => {
                        gl.uniform_matrix_3_f32_slice(Some(&location), false, v.as_flattened());
                    }
                    UniformValue::Mat4(v) => {
                        gl.uniform_matrix_4_f32_slice(Some(&location), false, v.as_flattened());
                    }
                    UniformValue::Int(v) => {
                        gl.uniform_1_i32(Some(&location), *v);
                    }
                    UniformValue::Bool(v) => {
                        gl.uniform_1_i32(Some(&location), *v as i32);
                    }
                    UniformValue::Sampler2D(tex) => {
                        tex.bind(0);
                    }
                    UniformValue::SamplerCube(tex_id) => {
                        todo!("Implement cubemap texture binding. Tried to bind {tex_id}");
                    }
                }
            }
        }
    }
}

impl Drop for GLProgram {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gl.as_ref();
            gl.delete_program(self.program);
        }
    }
}
