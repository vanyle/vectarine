use std::{cell::RefCell, collections::HashMap, path::Path, rc::Rc};

use tiled::Loader;

use crate::game_resource::{Resource, ResourceId, Status};
use vectarine_plugin_sdk::glow;

// MARK: Tileset

pub struct TilesetContent {
    pub tiled: tiled::Tileset,
    pub type_mapping: HashMap<Vec<u8>, u32>,
}

impl TilesetContent {
    pub fn from_tiled_tileset(tileset: tiled::Tileset) -> Self {
        let type_mapping = tileset
            .tiles()
            .filter_map(|(id, tile)| {
                tile.user_type
                    .as_ref()
                    .map(|t| (t.clone().into_bytes(), id))
            })
            .collect::<HashMap<_, _>>();
        Self {
            tiled: tileset,
            type_mapping,
        }
    }
}

pub struct TilesetResource {
    pub content: RefCell<Option<TilesetContent>>,
}

impl Resource for TilesetResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &super::DependencyReporter,
        _lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
        _gl: std::sync::Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let mut loader = Loader::with_reader(move |path: &std::path::Path| -> std::io::Result<_> {
            if path == std::path::Path::new("map") {
                Ok(std::io::Cursor::new(data.clone()))
            } else {
                Err(std::io::ErrorKind::NotFound.into())
            }
        });
        let tsx = loader.load_tsx_tileset(Path::new("map"));
        match tsx {
            Err(err) => Status::Error(err.to_string()),
            Ok(tileset) => {
                // self.content.replace(Some(Vec::from(data)));
                if tileset.image.is_none() {
                    Status::Error(
                        "No image tag inside the tileset. Only tileset based on an image are supported".to_string(),
                    )
                } else {
                    self.content
                        .replace(Some(TilesetContent::from_tiled_tileset(tileset)));
                    Status::Loaded
                }
            }
        }
    }

    fn draw_debug_gui(
        &self,
        _painter: &mut vectarine_plugin_sdk::egui_glow::Painter,
        ui: &mut vectarine_plugin_sdk::egui::Ui,
    ) {
        ui.label("Tileset Resource");
        let content = self.content.borrow();
        if let Some(data) = &*content {
            ui.label(format!("name: {}", data.tiled.name));
            ui.label(format!("tile width: {}", data.tiled.tile_width));
            ui.label(format!("tile height: {}", data.tiled.tile_height));
            ui.label(format!("tile count: {}", data.tiled.tilecount));
        } else {
            ui.label("<No content loaded>");
        }
    }

    fn get_type_name(&self) -> &'static str {
        "Tileset"
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

// MARK: Tilemap

pub struct TilemapResource {
    pub content: RefCell<Option<tiled::Map>>,
}

impl Resource for TilemapResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &super::DependencyReporter,
        _lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
        _gl: std::sync::Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let mut loader = Loader::with_reader(move |path: &std::path::Path| -> std::io::Result<_> {
            if path == std::path::Path::new("map") {
                Ok(std::io::Cursor::new(data.clone()))
            } else {
                Err(std::io::ErrorKind::NotFound.into())
            }
        });
        let tmx = loader.load_tmx_map(Path::new("map"));
        match tmx {
            Err(err) => Status::Error(err.to_string()),
            Ok(tilemap) => {
                self.content.replace(Some(tilemap));
                Status::Loaded
            }
        }
    }

    fn draw_debug_gui(
        &self,
        _painter: &mut vectarine_plugin_sdk::egui_glow::Painter,
        ui: &mut vectarine_plugin_sdk::egui::Ui,
    ) {
        ui.label("Tilemap Resource");
        let content = self.content.borrow();
        if let Some(data) = &*content {
            ui.label(format!("width: {}", data.width));
            ui.label(format!("height: {}", data.height));
            ui.label(format!("Layer count: {}", data.layers().len()));
        } else {
            ui.label("<No content loaded>");
        }
    }

    fn get_type_name(&self) -> &'static str {
        "Tilemap"
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
