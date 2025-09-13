use std::path::{Path, PathBuf};

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

    fn reload(&mut self);

    /// Draw an interface with information about the resource.
    fn draw_debug_gui(&mut self, ui: &mut egui::Ui);
    /// Create a resource from a file. If the resource has dependencies, load them too and
    /// store them in the ResourceManager.
    fn from_file(manager: &mut ResourceManager, path: &Path) -> Self
    where
        Self: Sized;
}
