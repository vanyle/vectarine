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
        let font_size = 64.0;
        let chars: Vec<char> = CHARSET.chars().collect();

        let mut char_data: Vec<(char, fontdue::Metrics, Vec<u8>)> = Vec::new();
        let mut total_width = 0u32;
        let mut max_height = 0u32;

        for &c in &chars {
            let (metrics, bitmap) = font.rasterize(c, font_size);
            total_width += metrics.width as u32;
            max_height = max_height.max(metrics.height as u32);
            char_data.push((c, metrics, bitmap));
        }

        const PADDING: u32 = 2;

        // Add some padding between characters and around the edges
        let atlas_width = total_width + ((chars.len() + 1) as u32 * PADDING);
        let atlas_height = max_height + PADDING * 2;

        // Create the atlas bitmap
        let mut atlas_data = vec![0u8; (atlas_width * atlas_height) as usize];
        let mut current_x = PADDING;

        // Initialize the font cache
        let mut font_cache = HashMap::new();

        // Second pass: place characters in atlas and store texture coordinates
        for (c, metrics, bitmap) in char_data {
            // Copy character bitmap to atlas
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
            font_size,
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
