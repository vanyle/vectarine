use std::{cell::RefCell, collections::HashMap, sync::Arc};

use crate::{
    graphics::gltexture,
    helpers::game_resource::{Resource, ResourceDescription},
};

#[derive(Debug, Clone)]
pub struct CharacterInfo {
    pub metrics: fontdue::Metrics,
    pub atlas_x: f32,      // Normalized texture coordinate (0-1) for left edge
    pub atlas_y: f32,      // Normalized texture coordinate (0-1) for top edge
    pub atlas_width: f32,  // Normalized width in atlas
    pub atlas_height: f32, // Normalized height in atlas
}

pub struct FontRenderingData {
    pub font_atlas: Arc<gltexture::Texture>,
    pub font_cache: HashMap<char, CharacterInfo>,
    pub font_loader: fontdue::Font,
    pub font_size: f32,
}

pub struct FontResource {
    pub description: ResourceDescription,
    pub font_rendering: RefCell<Option<FontRenderingData>>,
    pub is_loading: RefCell<bool>,
    pub error: RefCell<Option<String>>,
}

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:'\",.<>?/\\`~ \n";
const FONT_DETAIL: f32 = 64.0; // Base font size for rasterization

impl Resource for FontResource {
    fn get_resource_info(&self) -> ResourceDescription {
        self.description.clone()
    }

    fn reload_from_data(self: std::rc::Rc<Self>, gl: Arc<glow::Context>, data: Vec<u8>) {
        if data.is_empty() {
            self.is_loading.replace(false);
            self.error
                .replace(Some("File is empty or does not exist.".to_string()));
            return;
        }
        let font = fontdue::Font::from_bytes(data.as_slice(), fontdue::FontSettings::default());
        let font = match font {
            Err(e) => {
                self.is_loading.replace(false);
                self.error.replace(Some(e.to_string()));
                return;
            }
            Ok(f) => f,
        };

        self.is_loading.replace(false);
        self.error.replace(None);

        // Initialize the font atlas
        let chars: Vec<char> = CHARSET.chars().collect();

        let mut char_data: Vec<(char, fontdue::Metrics, Vec<u8>)> = Vec::new();
        let mut total_width = 0u32;
        let mut max_height = 0u32;

        for &c in &chars {
            let (metrics, bitmap) = font.rasterize(c, FONT_DETAIL);
            total_width += metrics.width as u32;
            max_height = max_height.max(metrics.height as u32);
            char_data.push((c, metrics, bitmap));
        }

        const PADDING: u32 = 2;

        // Add some padding between characters and around the edges
        let atlas_width = total_width + ((chars.len() + 1) as u32 * PADDING);
        let atlas_height = max_height + PADDING * 2;

        let mut atlas_data = vec![0u8; (atlas_width * atlas_height) as usize];
        let mut current_x = PADDING;

        let mut font_cache = HashMap::new();

        for (c, metrics, bitmap) in char_data {
            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    let src_idx = y * metrics.width + x;
                    let dst_idx =
                        (y + PADDING as usize) * atlas_width as usize + current_x as usize + x;

                    atlas_data[dst_idx] = bitmap[src_idx];
                }
            }

            // Calculate normalized texture coordinates for this character
            let atlas_x = current_x as f32 / atlas_width as f32;
            let atlas_y = 0 as f32 / atlas_height as f32;
            let atlas_width_norm = metrics.width as f32 / atlas_width as f32;
            let atlas_height_norm = metrics.height as f32 / atlas_height as f32;

            // Store character info with atlas coordinates
            let char_info = CharacterInfo {
                metrics,
                atlas_x,
                atlas_y,
                atlas_width: atlas_width_norm,
                atlas_height: atlas_height_norm,
            };
            font_cache.insert(c, char_info);

            current_x += metrics.width as u32 + PADDING;
        }

        // Create the OpenGL texture from the atlas
        let atlas_texture =
            gltexture::Texture::new_grayscale(&gl, &atlas_data, atlas_width, atlas_height);

        // Store the results
        self.font_rendering.replace(Some(FontRenderingData {
            font_atlas: atlas_texture,
            font_cache,
            font_loader: font,
            font_size: FONT_DETAIL,
        }));
    }

    fn draw_debug_gui(&self, ui: &mut egui::Ui) {
        ui.label("TODO: FontResource debug GUI");
    }

    fn get_loading_status(&self) -> super::ResourceStatus {
        if let Some(error) = self.error.borrow().as_ref() {
            return super::ResourceStatus::Error(error.clone());
        }
        if self.font_rendering.borrow().is_some() {
            super::ResourceStatus::Loaded
        } else if *self.is_loading.borrow() {
            super::ResourceStatus::Loading
        } else {
            super::ResourceStatus::Unloaded
        }
    }

    fn set_as_loading(&self) {
        self.is_loading.replace(true);
    }

    fn get_type_name(&self) -> &'static str {
        "FontResource"
    }

    fn from_file(_manager: &mut super::ResourceManager, path: &std::path::Path) -> Self
    where
        Self: Sized,
    {
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
            font_rendering: RefCell::new(None),
            is_loading: RefCell::new(false),
            error: RefCell::new(None),
        }
    }
}

impl FontResource {
    pub fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32, f32) {
        let font_rendering_data = self.font_rendering.borrow();
        let font_rendering_data = font_rendering_data.as_ref();
        let Some(font_rendering_data) = font_rendering_data else {
            return (0.0, 0.0, 0.0);
        };

        let mut width = 0.0;
        let mut height = 0.0;
        let mut max_ascent = 0.0;

        for c in text.chars() {
            if let Some(char_info) = font_rendering_data.font_cache.get(&c) {
                let bounds = char_info.metrics.bounds;
                width += char_info.metrics.advance_width;
                max_ascent = f32::max(max_ascent, bounds.height - bounds.ymin);
                height = f32::max(height, bounds.height);
            }
        }

        let scale = font_size / font_rendering_data.font_size;
        (width * scale, height * scale, max_ascent * scale)
    }
}
