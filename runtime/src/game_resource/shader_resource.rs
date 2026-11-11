use std::{cell::RefCell, path::Path, rc::Rc};

use crate::graphics::gfx::Gfx;
use crate::{
    game_resource::{DependencyReporter, Resource, ResourceId, Status},
    graphics::customshader::CustomShader,
    lua_env::LuaHandle,
};

pub struct ShaderResource {
    pub shader: RefCell<Option<CustomShader>>,
}

impl Resource for ShaderResource {
    fn get_type_name(&self) -> &'static str {
        "Shader"
    }
    fn load_from_data(
        self: Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &DependencyReporter,
        _lua: &Rc<LuaHandle>,
        gfx: Gfx,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let frag_src = match std::str::from_utf8(&data) {
            Ok(src) => src,
            Err(e) => {
                return Status::Error(format!("Shader is not valid UTF-8: {e}"));
            }
        };
        let shader = match CustomShader::from_source(gfx.gpu(), frag_src) {
            Ok(shader) => shader,
            Err(e) => {
                println!("Shader compilation error: {}", e);
                return Status::Error(format!("Failed to compile shader: {e}"));
            }
        };
        // The values set before a hot reload are kept: games often set a uniform only once.
        if let Some(previous) = self.shader.borrow().as_ref() {
            shader.copy_uniforms_from(previous);
        }
        self.shader.replace(Some(shader));

        Status::Loaded
    }

    fn draw_debug_gui(
        &self,
        _painter: &mut vectarine_plugin_sdk::egui_wgpu::Renderer,
        ui: &mut vectarine_plugin_sdk::egui::Ui,
    ) {
        ui.label("Shader Details:");
        let shader = self.shader.borrow();
        let Some(shader) = shader.as_ref() else {
            ui.label("No shader loaded.");
            return;
        };
        if shader.uniform_fields().is_empty() {
            ui.label("No uniforms.");
        } else {
            ui.label("Uniforms:");
            for field in shader.uniform_fields() {
                ui.label(format!("  {field}"));
            }
        }
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
