use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use crate::{
    game_resource::{DependencyReporter, Resource, ResourceId, Status},
    graphics::{
        glframebuffer, glprogram,
        gltexture::{self},
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
        _lua: &Rc<mlua::Lua>,
        _gl: Arc<glow::Context>,
        _path: &Path,
        data: &[u8],
    ) -> Status {
        if data.is_empty() {
            return Status::Error("File is empty or does not exist.".to_string());
        }

        // TODO: we want to same format as shadertoy for data, so we need to do some parsing and glsl transformation.

        Status::Loaded
    }

    fn draw_debug_gui(&self, ui: &mut egui::Ui) {
        ui.label("Shader Details:");
        let tex = self.shader.borrow();
        let Some(shader) = tex.as_ref() else {
            ui.label("No texture loaded.");
            return;
        };
        ui.label(format!("Layout: {}", shader.shader.uniform_layout));
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
