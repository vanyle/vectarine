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
    pub resources: Vec<Rc<dyn Resource>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceStatus {
    Loaded,
    Loading,
    Unloaded,
    Error,
}

impl std::fmt::Display for ResourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceStatus::Loaded => write!(f, "Loaded"),
            ResourceStatus::Loading => write!(f, "Loading"),
            ResourceStatus::Unloaded => write!(f, "Not yet loaded"),
            ResourceStatus::Error => write!(f, "Loading Error"),
        }
    }
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
        let resource = Rc::new(T::from_file(self, path));
        self.resources.push(resource);
        id
    }
}

pub trait Resource: ResourceToAny {
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

    /// A human-friendly name for this type of Resource.
    /// This is usually the name of the struct implementing the trait.
    fn get_type_name(&self) -> &'static str;

    /// Create a resource from a file. If the resource has dependencies, load them too and
    /// store them in the ResourceManager.
    fn from_file(manager: &mut ResourceManager, path: &Path) -> Self
    where
        Self: Sized;
}

pub fn get_absolute_path(resource_path: &Path) -> String {
    let abs_path = PathBuf::from("assets").join(resource_path);
    let abs_path = abs_path.canonicalize().unwrap_or(abs_path);
    let as_str = abs_path.to_string_lossy();
    as_str.into_owned()
}

pub trait ResourceToAny: 'static {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Resource + 'static> ResourceToAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
