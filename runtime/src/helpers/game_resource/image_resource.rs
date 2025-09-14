use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use image::metadata::Orientation;

use crate::{
    graphics::gltexture::{self, Texture},
    helpers::{
        file,
        game::Game,
        game_resource::{Resource, ResourceDescription, ResourceManager},
    },
};

pub struct ImageResource {
    pub description: ResourceDescription,
    pub texture: RefCell<Option<Arc<gltexture::Texture>>>,
    pub is_loading: RefCell<bool>,
    pub is_error: RefCell<bool>,
}

impl Resource for ImageResource {
    fn get_resource_info(&self) -> ResourceDescription {
        self.description.clone()
    }

    fn reload(self: Rc<Self>, gl: Arc<glow::Context>, _game: &mut Game) {
        let r = self.clone();
        self.is_loading.replace(true);

        let abs_path = PathBuf::from("assets").join(&self.description.path);
        let as_str = abs_path.to_string_lossy();

        file::read_file(
            &as_str,
            Box::new(move |data| {
                let result = image::load_from_memory(data.as_slice());
                let Ok(mut image) = result else {
                    r.is_loading.replace(false);
                    r.is_error.replace(true);
                    return;
                };
                // We could do this in the shader instead. I don't really know which option is better.
                image.apply_orientation(Orientation::FlipVertical);

                r.texture.replace(Some(Texture::new_rgba(
                    &gl,
                    image.to_rgba8().as_raw().as_slice(),
                    image.width(),
                    image.height(),
                )));
                r.is_loading.replace(false);
                r.is_error.replace(false);
            }),
        );
    }

    fn draw_debug_gui(&mut self, ui: &mut egui::Ui) {
        ui.label("[TODO Image resource interface]");
    }

    fn get_loading_status(&self) -> super::ResourceStatus {
        if self.texture.borrow().is_some() {
            super::ResourceStatus::Loaded
        } else if *self.is_loading.borrow() {
            super::ResourceStatus::Loading
        } else {
            super::ResourceStatus::Unloaded
        }
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
            is_error: RefCell::new(false),
        }
    }
}
