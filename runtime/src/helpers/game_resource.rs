use std::{
    cell::RefCell,
    collections::HashSet,
    ops::Index,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::Arc,
};

use crate::helpers::file;

pub mod font_resource;
pub mod image_resource;

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Unloaded,
    Loading,
    Loaded,
    Error(String),
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Unloaded => write!(f, "Not yet loaded"),
            Status::Loading => write!(f, "Loading"),
            Status::Loaded => write!(f, "Loaded"),
            Status::Error(msg) => write!(f, "Loading Error: {msg}"),
        }
    }
}

pub struct ResourceHolder {
    resource: Rc<dyn Resource>,
    status: RefCell<Status>,

    name: String,
    path: PathBuf,
    /// A list of ids of other resources that this resource needs to work
    dependencies: RefCell<HashSet<usize>>,
    /// A list of ids of other resources that depend on this resource
    dependent: RefCell<HashSet<usize>>,
}

impl ResourceHolder {
    /// Request the resource to be reloaded.
    fn reload(
        self: Rc<Self>,
        assigned_id: usize,
        resource_manager: Rc<ResourceManager>,
        gl: Arc<glow::Context>,
    ) {
        // Clean ourselves from dependent array of others:
        for dep_id in self.dependencies.borrow().iter() {
            if let Some(dep) = resource_manager.resources.borrow().get(*dep_id) {
                dep.dependent.borrow_mut().remove(&assigned_id);
            }
        }
        self.dependencies.borrow_mut().clear();

        let dr = DependencyReporter {
            resource_manager: Rc::downgrade(&resource_manager),
        };

        if self.is_loading() {
            return;
        }
        self.status.replace(Status::Loading);
        let abs_path = get_absolute_path(&self.path);

        file::read_file(
            &abs_path,
            Box::new(move |data| {
                let resulting_status =
                    self.resource
                        .clone()
                        .load_from_data(assigned_id, &dr, gl.clone(), &data);
                self.status.replace(resulting_status);
            }),
        );
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn draw_debug_gui(&self, ui: &mut egui::Ui) {
        self.resource.draw_debug_gui(ui);
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub fn get_type_name(&self) -> &'static str {
        self.resource.get_type_name()
    }

    pub fn get_status(&self) -> Status {
        self.status.borrow().clone()
    }

    pub fn is_loading(&self) -> bool {
        matches!(*self.status.borrow(), Status::Loading)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(*self.status.borrow(), Status::Loaded)
    }
}

#[derive(Default)]
pub struct ResourceManager {
    pub resources: RefCell<Vec<Rc<ResourceHolder>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceStatus {
    Loaded,
    Loading,
    Unloaded,
    Error(String),
}

impl std::fmt::Display for ResourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceStatus::Loaded => write!(f, "Loaded"),
            ResourceStatus::Loading => write!(f, "Loading"),
            ResourceStatus::Unloaded => write!(f, "Not yet loaded"),
            ResourceStatus::Error(msg) => write!(f, "Loading Error: {msg}"),
        }
    }
}

impl std::fmt::Debug for ResourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceManager")
            .field("resources_count", &self.resources.borrow().len())
            .finish()
    }
}

pub struct DependencyReporter {
    resource_manager: Weak<ResourceManager>,
}

impl DependencyReporter {
    /// Declare that the resource with id `resource_id` depends on the resource at `path`.
    /// This will trigger loading of the required dependencies.
    pub fn declare_dependency<T: Resource + 'static>(&self, id: usize, path: &Path) {
        let Some(resource_manager) = self.resource_manager.upgrade() else {
            return;
        };
        resource_manager.declare_dependency::<T>(id, path);
    }
}

impl ResourceManager {
    /// Create a new resource from a file and store it.
    pub fn load_resource<T: Resource + 'static>(&self, path: &Path) -> usize {
        let id = self.resources.borrow().len();
        let resource = Rc::new(T::default());
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        self.resources.borrow_mut().push(Rc::new(ResourceHolder {
            status: RefCell::new(Status::Unloaded),
            path: path.to_path_buf(),
            name,
            dependencies: RefCell::new(HashSet::new()),
            dependent: RefCell::new(HashSet::new()),
            resource,
        }));
        id
    }

    fn declare_dependency<T: Resource + 'static>(&self, resource_id: usize, path: &Path) {
        let resources = self.resources.borrow();
        let Some(resource) = resources.get(resource_id) else {
            unreachable!("Incorrect resource id {resource_id}");
        };
        // Check if the dependency is already exists. Create it if not.
        let holder = &self
            .get_id_by_path(path)
            .map(|id| self.get_holder_by_id_unchecked(id));
        if let Some(holder) = holder {
            holder.dependent.borrow_mut().insert(resource_id);
            resource.dependent.borrow_mut().insert(resource_id);
            return;
        };
        self.load_resource::<T>(path);
    }

    pub fn reload(self: &Rc<Self>, id: usize, gl: Arc<glow::Context>) {
        let resource = self.resources.borrow()[id].clone();
        resource.reload(id, self.clone(), gl);
    }

    pub fn get_id_by_path(&self, path: &Path) -> Option<usize> {
        let to_match = get_absolute_path(path);
        for (i, res) in self.resources.borrow().iter().enumerate() {
            let p = get_absolute_path(&res.path);
            if to_match == p {
                return Some(i);
            }
        }
        None
    }

    pub fn get_by_id<T: Resource + 'static>(&self, id: usize) -> Result<Rc<T>, String> {
        let resources = self.resources.borrow();
        let resource = resources.get(id).ok_or("Resource not found")?;
        if !resource.is_loaded() {
            return Err("Resource not available yet".into());
        }
        let res = resource.resource.clone().as_any_rc();
        let res = res.downcast::<T>().map_err(|_| "Resource type mismatch")?;
        Ok(res)
    }

    pub fn get_holder_by_id(&self, id: usize) -> Option<Rc<ResourceHolder>> {
        let resources = self.resources.borrow();
        resources.get(id).cloned()
    }

    pub fn get_holder_by_id_unchecked(&self, id: usize) -> Rc<ResourceHolder> {
        let resources = self.resources.borrow();
        (*resources).index(id).clone()
    }

    pub fn get_by_path(&self, path: &Path) -> Option<Rc<dyn Resource>> {
        let to_match = get_absolute_path(path);

        for res in self.resources.borrow().iter() {
            let p1 = get_absolute_path(&res.path);
            if to_match == p1 {
                return Some(res.resource.clone());
            }
        }
        None
    }
}

/// Represents a resource, a dependency on external data that can be loaded and used by the game.
/// Usually, resources are implemented as struct with a RefCell<Option<T>>.
/// Resources can have dependencies.
pub trait Resource: ResourceToAny {
    /// Load the resource from the data and initialize it.
    /// It can call the resource manager to declare dependencies.
    /// If the loading is successful, return `Loaded``.
    /// If the loading failed, return `Error`` with a message.
    /// If the resource did not load because it needs dependencies which are not yet loaded, return `Unloaded`.
    /// If the resource wants to prevent any further loading attempts, return `Loading` (this should be rare).
    fn load_from_data(
        self: Rc<Self>,
        assigned_id: usize,
        dependency_reporter: &DependencyReporter,
        gl: Arc<glow::Context>,
        data: &[u8],
    ) -> Status;

    /// Draw an interface with information about the resource.
    fn draw_debug_gui(&self, ui: &mut egui::Ui);

    /// A human-friendly name for this type of Resource.
    /// This is usually the name of the struct implementing the trait.
    fn get_type_name(&self) -> &'static str;

    /// Create an empty instance of a resource
    fn default() -> Self
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
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn std::any::Any>;
}

impl<T: Resource + 'static> ResourceToAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}
