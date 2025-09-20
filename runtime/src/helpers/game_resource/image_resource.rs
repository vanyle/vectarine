use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use image::metadata::Orientation;

use crate::{
    graphics::gltexture::{self, Texture},
    helpers::game_resource::{DependencyReporter, Resource, ResourceId, Status},
};

pub struct ImageResource {
    pub texture: RefCell<Option<Arc<gltexture::Texture>>>,
}

impl Resource for ImageResource {
    fn get_type_name(&self) -> &'static str {
        "Image"
    }
    fn load_from_data(
        self: Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &DependencyReporter,
        _lua: Rc<mlua::Lua>,
        gl: Arc<glow::Context>,
        _path: &Path,
        data: &[u8],
    ) -> Status {
        if data.is_empty() {
            return Status::Error("File is empty or does not exist.".to_string());
        }
        let result = image::load_from_memory(data);
        let Ok(mut image) = result else {
            return Status::Error(format!("{}", result.err().unwrap()));
        };
        // We could do this in the shader instead. I don't really know which option is better.
        image.apply_orientation(Orientation::FlipVertical);

        self.texture.replace(Some(Texture::new_rgba(
            &gl,
            image.to_rgba8().as_raw().as_slice(),
            image.width(),
            image.height(),
        )));
        Status::Loaded
    }

    fn draw_debug_gui(&self, ui: &mut egui::Ui) {
        ui.label("Image Details:");
        let tex = self.texture.borrow();
        let Some(tex) = tex.as_ref() else {
            ui.label("No texture loaded.");
            return;
        };
        ui.label(format!("Width: {}", tex.width()));
        ui.label(format!("Height: {}", tex.height()));
        ui.label(format!("OpenGL ID: {}", tex.id()));
    }

    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            texture: RefCell::new(None),
        }
    }
}
