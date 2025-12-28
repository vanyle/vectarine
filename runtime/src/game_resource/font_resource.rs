use lazy_static::lazy_static;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    game_resource::{DependencyReporter, Resource, ResourceId, Status},
    graphics::gltexture,
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
    pub font_rendering: RefCell<Option<FontRenderingData>>,
}

pub fn use_default_font<F, R>(gl: &Arc<glow::Context>, f: F) -> R
where
    F: FnOnce(&mut FontRenderingData) -> R,
{
    lazy_static! {
        static ref DEFAULT_FONT: Mutex<Option<FontRenderingData>> = Mutex::new(None);
    }

    let mut default_font = DEFAULT_FONT
        .lock()
        .expect("Failed to acquire lock on the default font.");

    if let Some(default_font) = default_font.as_mut() {
        return f(default_font);
    }

    let font_bytes = include_bytes!("../../../assets/Roboto-Regular.ttf");
    let font = fontdue::Font::from_bytes(font_bytes.as_ref(), fontdue::FontSettings::default())
        .expect("The default font file contains a valid font.");
    let chars: Vec<char> = CHARSET.chars().collect();
    let (atlas_texture, font_cache) = initialize_cache_and_texture(gl, &font, chars);
    let mut font = FontRenderingData {
        font_atlas: atlas_texture,
        font_cache,
        font_loader: font,
        font_size: FONT_DETAIL,
    };
    let result = f(&mut font);
    *default_font = Some(font);
    result
}

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:'\",.<>?/\\`~ \n";
const FONT_DETAIL: f32 = 64.0; // Base font size for rasterization

impl Resource for FontResource {
    fn load_from_data(
        self: Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &DependencyReporter,
        _lua: &Rc<mlua::Lua>,
        gl: Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default());
        let font = match font {
            Err(e) => {
                return Status::Error(e.to_string());
            }
            Ok(f) => f,
        };

        // Initialize the font atlas
        let chars: Vec<char> = CHARSET.chars().collect();
        let (atlas_texture, font_cache) = initialize_cache_and_texture(&gl, &font, chars);

        // Store the results
        self.font_rendering.replace(Some(FontRenderingData {
            font_atlas: atlas_texture,
            font_cache,
            font_loader: font,
            font_size: FONT_DETAIL,
        }));
        Status::Loaded
    }

    fn draw_debug_gui(&self, _painter: &mut egui_glow::Painter, ui: &mut egui::Ui) {
        let font_data = self.font_rendering.borrow();
        let font_data = font_data.as_ref();
        let Some(font_data) = font_data else {
            ui.label("Font not loaded");
            return;
        };
        ui.label(format!(
            "Underlying texture atlas: {:?}",
            font_data.font_atlas.id()
        ));

        ui.label(format!("Font size: {:?}", font_data.font_size));
        ui.label(format!(
            "Available glyph count: {:?}",
            font_data.font_cache.len()
        ));
    }

    fn get_type_name(&self) -> &'static str {
        "Font"
    }

    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            font_rendering: RefCell::new(None),
        }
    }
}

impl FontRenderingData {
    pub fn measure_text(&self, text: &str, font_size: f32, aspect_ratio: f32) -> (f32, f32, f32) {
        let mut width = 0.0;
        let mut height = 0.0;
        let mut max_ascent = 0.0;

        for c in text.chars() {
            if let Some(char_info) = self.font_cache.get(&c) {
                let bounds = char_info.metrics.bounds;
                width += char_info.metrics.advance_width;
                max_ascent = f32::max(max_ascent, bounds.height - bounds.ymin);
                height = f32::max(height, bounds.height);
            }
        }

        let scale = font_size / self.font_size;
        (
            width * scale / aspect_ratio,
            height * scale,
            max_ascent * scale,
        )
    }

    /// Given some text, rebuild the atlas to include any missing character from the text.
    /// This function can be expensive, so try to use it rarely.
    pub fn enrich_atlas(&mut self, gl: &Arc<glow::Context>, text: &str) {
        // Note: we use chars and not glyphs. Support for non-Latin languages should not be too hard to add though.
        let mut chars_to_include: HashSet<char> = self.font_cache.keys().cloned().collect();
        let initial_character_count = chars_to_include.len();
        for c in text.chars() {
            chars_to_include.insert(c);
        }

        if initial_character_count == chars_to_include.len() {
            return;
        }

        let (atlas_texture, font_cache) =
            initialize_cache_and_texture(gl, &self.font_loader, chars_to_include);

        // Store the results
        self.font_atlas = atlas_texture;
        self.font_cache = font_cache;
    }
}

fn initialize_cache_and_texture(
    gl: &Arc<glow::Context>,
    font: &fontdue::Font,
    chars: impl IntoIterator<Item = char>,
) -> (Arc<gltexture::Texture>, HashMap<char, CharacterInfo>) {
    let mut char_data: Vec<(char, fontdue::Metrics, Vec<u8>)> = Vec::new();
    let mut total_width = 0u32;
    let mut max_height = 0u32;

    for c in chars {
        let (metrics, bitmap) = font.rasterize(c, FONT_DETAIL);
        total_width += metrics.width as u32;
        max_height = max_height.max(metrics.height as u32);
        char_data.push((c, metrics, bitmap));
    }

    const PADDING: u32 = 2;

    // Add some padding between characters and around the edges
    let atlas_width = total_width + ((char_data.len() + 1) as u32 * PADDING);
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
        gltexture::Texture::new_grayscale(gl, &atlas_data, atlas_width, atlas_height);

    (atlas_texture, font_cache)
}
