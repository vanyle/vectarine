use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceDescription {
    pub id: u32,
    pub name: String,
    pub path: PathBuf,
    pub dependencies: Vec<u32>,
}

pub struct ResourceManager {
    pub resources: Vec<Box<dyn Resource>>,
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
    fn draw_debug_gui(&mut self);
    /// Create a resource from a file. If the resource has dependencies, load them too and
    /// store them in the ResourceManager.
    fn from_file(manager: &mut ResourceManager, path: &Path) -> Self
    where
        Self: Sized;
}
