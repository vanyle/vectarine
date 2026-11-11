use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use crate::graphics::gfx::Gfx;
use crate::{
    game_resource::{DependencyReporter, Resource, ResourceId, Status},
    graphics::{
        gpu::Gpu,
        texture::{self, ImageAntialiasing, Texture},
    },
    lua_env::LuaHandle,
};

pub struct ImageResource {
    pub texture: RefCell<Option<Arc<texture::Texture>>>,
    pub egui_id: RefCell<Option<vectarine_plugin_sdk::egui::TextureId>>,
    pub antialiasing: Option<ImageAntialiasing>,
    /// The graphics card that holds the texture
    pub gpu: RefCell<Option<Arc<Gpu>>>,
}

impl Resource for ImageResource {
    fn get_type_name(&self) -> &'static str {
        "Image"
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
        let result = image::load_from_memory(&data);
        let image = match result {
            Err(err) => return Status::Error(format!("{}", err)),
            Ok(image) => image,
        };

        self.texture.replace(Some(Texture::new_rgba(
            gfx.gpu(),
            Some(image.to_rgba8().as_raw().as_slice()),
            image.width(),
            image.height(),
            self.antialiasing.unwrap_or(ImageAntialiasing::Linear),
        )));
        self.egui_id.replace(None);
        self.gpu.replace(Some(gfx.gpu().clone()));
        Status::Loaded
    }

    fn draw_debug_gui(
        &self,
        painter: &mut vectarine_plugin_sdk::egui_wgpu::Renderer,
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

        let gpu = self.gpu.borrow();
        let Some(gpu) = gpu.as_ref() else {
            return;
        };
        let mut egui_id = self.egui_id.borrow_mut();
        let texture_id = match egui_id.as_mut() {
            Some(id) => *id,
            None => {
                let native_tex = painter.register_native_texture(
                    &gpu.device,
                    tex.view(),
                    vectarine_plugin_sdk::wgpu::FilterMode::Linear,
                );
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
            gpu: RefCell::new(None),
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
