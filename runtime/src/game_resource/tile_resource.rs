use std::{cell::RefCell, path::Path, rc::Rc};

use tiled::Loader;

use crate::game_resource::{Resource, ResourceId, Status};

pub struct TilesetResource {
    pub content: RefCell<Option<tiled::Tileset>>,
}

impl Resource for TilesetResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &super::DependencyReporter,
        _lua: &Rc<mlua::Lua>,
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
                    self.content.replace(Some(tileset));
                    Status::Loaded
                }
            }
        }
    }

    fn draw_debug_gui(&self, ui: &mut egui::Ui) {
        ui.label("Tileset Resource");
        let content = self.content.borrow();
        if let Some(data) = &*content {
            ui.label(format!("name: {}", data.name));
            ui.label(format!("tile width: {}", data.tile_width));
            ui.label(format!("tile height: {}", data.tile_height));
            ui.label(format!("tile count: {}", data.tilecount));
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
