use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use crate::{
    game_resource::{DependencyReporter, Resource, ResourceId, Status},
    graphics::{
        glprogram,
        gltypes::{DataLayout, GLTypes, UsageHint},
    },
};

pub struct Shader {
    pub shader: glprogram::GLProgram,
}

// Fragment-shader is user-provided.
const BASE_VERTEX_SHADER: &str = r#"
layout (location = 0) in vec3 in_vert;
layout (location = 1) in vec2 in_uv;
out vec2 uv;
void main() {
    uv = in_uv;
    gl_Position = vec4(in_vert.xyz, 1.0);
}"#;

pub struct ShaderResource {
    pub shader: RefCell<Option<Shader>>,
}

impl Resource for ShaderResource {
    fn get_type_name(&self) -> &'static str {
        "Shader"
    }
    fn load_from_data(
        self: Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &DependencyReporter,
        _lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
        gl: Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let frag_src = match std::str::from_utf8(&data) {
            Ok(src) => src,
            Err(e) => {
                return Status::Error(format!("Shader is not valid UTF-8: {e}"));
            }
        };
        let program = glprogram::GLProgram::from_source(&gl, BASE_VERTEX_SHADER, frag_src);
        let mut program = match program {
            Ok(p) => p,
            Err(e) => {
                println!("Shader compilation error: {}", e);
                return Status::Error(format!("Failed to compile shader: {e}"));
            }
        };
        let mut layout = DataLayout::new();
        layout.add_field("in_vert", GLTypes::Vec2, Some(UsageHint::Position));
        layout.add_field("in_uv", GLTypes::Vec2, Some(UsageHint::TexCoord));
        program.vertex_layout = layout;
        self.shader.replace(Some(Shader { shader: program }));

        Status::Loaded
    }

    fn draw_debug_gui(&self, _painter: &mut vectarine_plugin_sdk::egui_glow::Painter, ui: &mut vectarine_plugin_sdk::egui::Ui) {
        ui.label("Shader Details:");
        let tex = self.shader.borrow();
        let Some(shader) = tex.as_ref() else {
            ui.label("No texture loaded.");
            return;
        };
        ui.label(format!("Layout: {}", shader.shader.vertex_layout));
    }

    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            shader: RefCell::new(None),
        }
    }
}
