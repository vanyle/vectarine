use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use image::metadata::Orientation;

use crate::{
    graphics::gltexture::{self, Texture},
    helpers::game_resource::{Resource, ResourceDescription, ResourceManager},
};

pub struct ImageResource {
    pub description: ResourceDescription,
    pub texture: RefCell<Option<Arc<gltexture::Texture>>>,
    pub is_loading: RefCell<bool>,
    pub error: RefCell<Option<String>>,
}

impl Resource for ImageResource {
    fn get_type_name(&self) -> &'static str {
        "ImageResource"
    }
    fn get_resource_info(&self) -> ResourceDescription {
        self.description.clone()
    }
    fn reload_from_data(self: Rc<Self>, gl: Arc<glow::Context>, data: Vec<u8>) {
        if data.is_empty() {
            self.is_loading.replace(false);
            self.error
                .replace(Some("File is empty or does not exist.".to_string()));
            return;
        }
        let result = image::load_from_memory(data.as_slice());
        let Ok(mut image) = result else {
            self.is_loading.replace(false);
            self.error
                .replace(Some(format!("{}", result.err().unwrap())));
            return;
        };
        // We could do this in the shader instead. I don't really know which option is better.
        image.apply_orientation(Orientation::FlipVertical);

        self.texture.replace(Some(Texture::new_rgba(
            &gl,
            image.to_rgba8().as_raw().as_slice(),
            image.width(),
            image.height(),
        )));
        self.is_loading.replace(false);
        self.error.replace(None);
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

    fn get_loading_status(&self) -> super::ResourceStatus {
        if let Some(error) = self.error.borrow().as_ref() {
            return super::ResourceStatus::Error(error.clone());
        }
        if self.texture.borrow().is_some() {
            super::ResourceStatus::Loaded
        } else if *self.is_loading.borrow() {
            super::ResourceStatus::Loading
        } else {
            super::ResourceStatus::Unloaded
        }
    }

    fn set_as_loading(&self) {
        self.is_loading.replace(true);
        self.error.replace(None);
    }

    fn from_file(_manager: &mut ResourceManager, path: &Path) -> Self {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            description: ResourceDescription {
                name,
                path: path.to_path_buf(),
                dependencies: Vec::new(),
            },
            texture: RefCell::new(None),
            is_loading: RefCell::new(false),
            error: RefCell::new(None),
        }
    }
}
