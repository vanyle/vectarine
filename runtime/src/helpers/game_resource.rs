use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use crate::helpers::game::Game;

pub mod image_resource;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceDescription {
    /// A non-unique human readable name for the resource
    pub name: String,
    /// A path to where the resource in stored in file form
    pub path: PathBuf,
    /// A list of ids of other resources that this resource needs to work
    pub dependencies: Vec<u32>,
}

#[derive(Default)]
pub struct ResourceManager {
    pub resources: Vec<Box<dyn Resource>>,
}

pub enum ResourceStatus {
    Loaded,
    Loading,
    Unloaded,
    Error,
}

impl std::fmt::Debug for ResourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceManager")
            .field("resources_count", &self.resources.len())
            .finish()
    }
}

impl ResourceManager {
    /// Create a new resource from a file and store it.
    pub fn load_resource<T: Resource + 'static>(&mut self, path: &Path) -> u32 {
        let id = self.resources.len() as u32;
        let resource = Box::new(T::from_file(self, path));
        self.resources.push(resource);
        id
    }
}

pub trait Resource {
    fn get_resource_info(&self) -> ResourceDescription;

    fn reload(self: Rc<Self>, gl: Arc<glow::Context>, game: &mut Game);

    /// Draw an interface with information about the resource.
    fn draw_debug_gui(&mut self, ui: &mut egui::Ui);

    /// A resource can be in an unloaded state. If that is true, reload will be called until the resource is loaded or loading.
    /// A resource is loaded if it is in a usable state.
    fn get_loading_status(&self) -> ResourceStatus;

    fn is_loading(&self) -> bool {
        matches!(self.get_loading_status(), ResourceStatus::Loading)
    }

    fn is_loaded(&self) -> bool {
        matches!(self.get_loading_status(), ResourceStatus::Loaded)
    }

    /// Create a resource from a file. If the resource has dependencies, load them too and
    /// store them in the ResourceManager.
    fn from_file(manager: &mut ResourceManager, path: &Path) -> Self
    where
        Self: Sized;
}
