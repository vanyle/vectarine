use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use crate::{
    game_resource::{DependencyReporter, Resource, ResourceId, Status},
    graphics::gltexture::{self, ImageAntialiasing, Texture},
};

pub struct ImageResource {
    pub texture: RefCell<Option<Arc<gltexture::Texture>>>,
    pub egui_id: RefCell<Option<vectarine_plugin_sdk::egui::TextureId>>,
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
        _lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
        gl: Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let result = image::load_from_memory(&data);
        let image = match result {
            Err(err) => return Status::Error(format!("{}", err)),
            Ok(image) => image,
        };

        self.texture.replace(Some(Texture::new_rgba(
            &gl,
            Some(image.to_rgba8().as_raw().as_slice()),
            image.width(),
            image.height(),
            self.antialiasing.unwrap_or(ImageAntialiasing::Linear),
        )));
        self.egui_id.replace(None);
        Status::Loaded
    }

    fn draw_debug_gui(
        &self,
        painter: &mut vectarine_plugin_sdk::egui_glow::Painter,
        ui: &mut vectarine_plugin_sdk::egui::Ui,
    ) {
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

        let mut egui_id = self.egui_id.borrow_mut();
        let texture_id = match egui_id.as_mut() {
            Some(id) => *id,
            None => {
                let native_tex = painter.register_native_texture(tex.id());
                *egui_id = Some(native_tex);
                native_tex
            }
        };

        let sized_texture = vectarine_plugin_sdk::egui::load::SizedTexture::new(
            texture_id,
            vectarine_plugin_sdk::egui::vec2(tex.width() as f32, tex.height() as f32),
        );
        let size = get_desired_size(
            vectarine_plugin_sdk::egui::vec2(tex.width() as f32, tex.height() as f32),
            200.0,
            200.0,
        );

        let image = vectarine_plugin_sdk::egui::Image::from_texture(sized_texture)
            .max_size(size)
            .corner_radius(5);
        ui.add(image);
    }

    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            texture: RefCell::new(None),
            egui_id: RefCell::new(None),
            antialiasing: None,
        }
    }
}

/// Preserves the aspect ratio of the image.
fn get_desired_size(
    actual_size: vectarine_plugin_sdk::egui::Vec2,
    max_width: f32,
    max_height: f32,
) -> vectarine_plugin_sdk::egui::Vec2 {
    let width_scale = max_width / actual_size.x;
    let height_scale = max_height / actual_size.y;
    let scale = width_scale.min(height_scale);
    vectarine_plugin_sdk::egui::Vec2::new(actual_size.x * scale, actual_size.y * scale)
}
