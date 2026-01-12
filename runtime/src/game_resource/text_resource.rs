use std::{cell::RefCell, path::Path, rc::Rc};

use crate::game_resource::{Resource, ResourceId, Status};

/// The most simple resource, a .txt file with some content.
pub struct TextResource {
    pub content: RefCell<Option<Vec<u8>>>,
}

impl Resource for TextResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &super::DependencyReporter,
        _lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
        _gl: std::sync::Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        self.content.replace(Some(Vec::from(data)));
        Status::Loaded
    }

    fn draw_debug_gui(&self, _painter: &mut vectarine_plugin_sdk::egui_glow::Painter, ui: &mut vectarine_plugin_sdk::egui::Ui) {
        ui.label("Text Resource");
        let content = self.content.borrow();
        if let Some(data) = &*content {
            if let Ok(text) = std::str::from_utf8(data) {
                ui.text_edit_multiline(&mut text.to_string());
            } else {
                ui.label("<Non-UTF8 content>");
            }
        } else {
            ui.label("<No content loaded>");
        }
    }

    fn get_type_name(&self) -> &'static str {
        "Text"
    }

    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            content: RefCell::new(None),
        }
    }
}
