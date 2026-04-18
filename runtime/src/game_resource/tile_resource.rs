use std::{cell::RefCell, collections::HashMap, path::Path, rc::Rc, sync::Arc};

use tiled::{DefaultResourceCache, Loader, ResourceCache};

use crate::{
    game_resource::{Resource, ResourceId, Status},
    lua_env::LuaHandle,
};
use vectarine_plugin_sdk::glow;

// MARK: Tileset

pub struct TilesetContent {
    pub tiled: Arc<tiled::Tileset>,
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
            tiled: Arc::new(tileset),
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
        _lua: &Rc<LuaHandle>,
        _gl: std::sync::Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let cache = DefaultResourceCache::new();

        let mut loader = Loader::with_cache_and_reader(
            cache,
            move |path: &std::path::Path| -> std::io::Result<_> {
                if path == std::path::Path::new("map") {
                    Ok(std::io::Cursor::new(data.clone()))
                } else {
                    Err(std::io::ErrorKind::NotFound.into())
                }
            },
        );
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

// MARK: Cache

struct VectarineResourceCacheData<'a> {
    dependency_reporter: &'a super::DependencyReporter,
    everything_was_found: RefCell<bool>,
    resource_id: ResourceId,
}
#[derive(Clone)]
struct VectarineResourceCache<'a>(Rc<VectarineResourceCacheData<'a>>);

impl<'a> VectarineResourceCache<'a> {
    pub fn new(
        dependency_reporter: &'a super::DependencyReporter,
        resource_id: ResourceId,
    ) -> Self {
        Self(Rc::new(VectarineResourceCacheData {
            dependency_reporter,
            everything_was_found: RefCell::new(true),
            resource_id,
        }))
    }
}

impl ResourceCache for VectarineResourceCache<'_> {
    fn get_tileset(
        &self,
        path: impl AsRef<tiled::ResourcePath>,
    ) -> Option<std::sync::Arc<tiled::Tileset>> {
        let Some(resource_id) = self.0.dependency_reporter.obtain_resource_id(path.as_ref()) else {
            *self.0.everything_was_found.borrow_mut() = false;
            // The resource does not exist (it is not even a matter of being loaded or not), so we report it as a dependency to start loading it.
            self.0
                .dependency_reporter
                .declare_dependency::<TilesetResource>(self.0.resource_id, path.as_ref());
            return None;
        };
        let Ok(resource) = self
            .0
            .dependency_reporter
            .obtain_resource::<TilesetResource>(&resource_id)
        else {
            // Here, we could need to distinguish between resource is not loaded / an error occured during loading.
            // If there is an error with the dependencies, trying to load the current resource is not useful.
            // We still flag it as loading anyway.
            *self.0.everything_was_found.borrow_mut() = false;
            return None;
        };
        let content = resource.content.borrow();
        let content = content.as_ref()?;
        Some(content.tiled.clone())
    }

    fn insert_tileset(
        &mut self,
        _path: impl AsRef<tiled::ResourcePath>,
        _tileset: std::sync::Arc<tiled::Tileset>,
    ) {
    }

    fn get_template(
        &self,
        _path: impl AsRef<tiled::ResourcePath>,
    ) -> Option<std::sync::Arc<tiled::Template>> {
        // We don't support templates for now.
        None
    }

    fn insert_template(
        &mut self,
        _path: impl AsRef<tiled::ResourcePath>,
        _tileset: std::sync::Arc<tiled::Template>,
    ) {
    }
}

// MARK: Tilemap

pub struct TilemapResource {
    pub content: RefCell<Option<tiled::Map>>,
}

impl Resource for TilemapResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        assigned_id: ResourceId,
        dependency_reporter: &super::DependencyReporter,
        _lua: &Rc<LuaHandle>,
        _gl: std::sync::Arc<glow::Context>,
        tilemap_path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let cache = VectarineResourceCache::new(dependency_reporter, assigned_id);

        let mut loader = Loader::with_cache_and_reader(
            cache.clone(),
            move |path: &std::path::Path| -> std::io::Result<_> {
                if path == tilemap_path {
                    return Ok(std::io::Cursor::new(data.clone()));
                }
                // It is too late to get the data of something else, it should be in the cache if it is a dependency, otherwise it is missing
                Err(std::io::ErrorKind::NotFound.into())
            },
        );

        let every_dependency_is_available = *cache.0.everything_was_found.borrow();
        if !every_dependency_is_available {
            return Status::Loading;
        }

        let tmx = loader.load_tmx_map(tilemap_path);
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
