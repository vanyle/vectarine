use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use crate::{
    game_resource::{DependencyReporter, Resource, ResourceId, Status},
    graphics::gltexture::{self, ImageAntialiasing, Texture},
};

pub struct ImageResource {
    pub texture: RefCell<Option<Arc<gltexture::Texture>>>,
    pub antialiasing: Option<ImageAntialiasing>,
}

impl Resource for ImageResource {
    fn get_type_name(&self) -> &'static str {
        "Image"
    }
    fn load_from_data(
        self: Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &DependencyReporter,
        _lua: &Rc<mlua::Lua>,
        gl: Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let result = image::load_from_memory(&data);
        let Ok(image) = result else {
            return Status::Error(format!("{}", result.err().unwrap()));
        };

        self.texture.replace(Some(Texture::new_rgba(
            &gl,
            Some(image.to_rgba8().as_raw().as_slice()),
            image.width(),
            image.height(),
            self.antialiasing.unwrap_or(ImageAntialiasing::Linear),
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
        ui.label(format!("Antialiasing: {:?}", self.antialiasing));
        ui.label(format!("OpenGL ID: {}", tex.id().0));
    }

    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            texture: RefCell::new(None),
            antialiasing: None,
        }
    }
}
