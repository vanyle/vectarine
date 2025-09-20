use std::{
    cell::RefCell,
    collections::HashSet,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::helpers::file;

pub mod font_resource;
pub mod image_resource;
pub mod script_resource;

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
            Status::Error(msg) => write!(f, "Error: {msg}"),
        }
    }
}

/// Represents a valid identifier for a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(usize);

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RID({})", self.0)
    }
}

impl mlua::FromLua for ResourceId {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "ResourceId".to_string(),
                message: Some("Expected ResourceId".to_string()),
            }),
        }
    }
}

impl mlua::UserData for ResourceId {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::ToString, |_lua, (id,): (Self,)| {
            Ok(format!("ResourceId({})", id.0))
        });
    }
}

pub struct ResourceHolder {
    resource: Rc<dyn Resource>,
    status: RefCell<Status>,

    name: String,
    path: PathBuf,
    /// A list of ids of other resources that this resource needs to work
    dependencies: RefCell<HashSet<ResourceId>>,
    /// A list of ids of other resources that depend on this resource
    dependent: RefCell<HashSet<ResourceId>>,
}

impl ResourceHolder {
    /// Request the resource to be reloaded.
    fn reload(
        self: Rc<Self>,
        assigned_id: ResourceId,
        resource_manager: Rc<ResourceManager>,
        lua: Rc<mlua::Lua>,
        gl: Arc<glow::Context>,
    ) {
        // Clean ourselves from dependent array of others:
        for dep_id in self.dependencies.borrow().iter() {
            let dep = resource_manager.get_holder_by_id(*dep_id);
            dep.dependent.borrow_mut().remove(&assigned_id);
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

        // We pass data to the resource into the closure.
        // As this data needs to be kept alive, every piece of state pass inside needs Rc or Arc.
        file::read_file(
            &abs_path,
            Box::new(move |data| {
                let Some(data) = data else {
                    self.status.replace(Status::Error(format!(
                        "File not found: {}",
                        self.path.display()
                    )));
                    return;
                };
                let resulting_status = self.resource.clone().load_from_data(
                    assigned_id,
                    &dr,
                    lua.clone(),
                    gl.clone(),
                    &self.path,
                    &data,
                );
                self.status.replace(resulting_status);
            }),
        );
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_underlying_resource<T: Resource + 'static>(&self) -> Result<Rc<T>, String> {
        let res = self.resource.clone().as_any_rc();
        let res = res.downcast::<T>().map_err(|_| {
            format!(
                "Resource type mismatch, {} expected, {} found",
                std::any::type_name::<T>(),
                std::any::type_name::<Self>()
            )
        })?;
        Ok(res)
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
    resources: RefCell<Vec<Rc<ResourceHolder>>>,
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
            ResourceStatus::Error(msg) => write!(f, "Error: {msg}"),
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
    pub fn declare_dependency<T: Resource + 'static>(&self, id: ResourceId, path: &Path) {
        let Some(resource_manager) = self.resource_manager.upgrade() else {
            return;
        };
        resource_manager.declare_dependency::<T>(id, path);
    }
}

impl ResourceManager {
    /// Create a new resource from a file and store it.
    /// If the resource already exists at that path, do nothing.
    /// Return the id of the resource.
    pub fn load_resource<T: Resource + 'static>(&self, path: &Path) -> ResourceId {
        if let Some(id) = self.get_id_by_path(path) {
            return id;
        }
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
        ResourceId(id)
    }

    fn declare_dependency<T: Resource + 'static>(&self, resource_id: ResourceId, path: &Path) {
        let resources = self.resources.borrow();
        let Some(resource) = resources.get(resource_id.0) else {
            unreachable!("Incorrect resource id {}", resource_id.0);
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

    pub fn reload(self: &Rc<Self>, id: ResourceId, lua: Rc<mlua::Lua>, gl: Arc<glow::Context>) {
        let resource = self.get_holder_by_id(id);
        resource.reload(id, self.clone(), lua, gl);
    }

    /// Performance: O(n) for now. Store the ID and use instead get_by_id if you already have the id.
    /// instead of get_by_path.
    pub fn get_id_by_path(&self, path: &Path) -> Option<ResourceId> {
        let to_match = get_absolute_path(path);
        for (i, res) in self.resources.borrow().iter().enumerate() {
            let p = get_absolute_path(&res.path);
            if to_match == p {
                return Some(ResourceId(i));
            }
        }
        None
    }

    pub fn get_by_id<T: Resource + 'static>(&self, id: ResourceId) -> Result<Rc<T>, String> {
        let resource = self.get_holder_by_id(id);
        if !resource.is_loaded() {
            return Err("Resource not available yet".into());
        }
        resource.get_underlying_resource::<T>()
    }

    pub fn get_holder_by_id(&self, id: ResourceId) -> Rc<ResourceHolder> {
        let resources = self.resources.borrow();
        match resources.get(id.0) {
            Some(res) => res.clone(),
            None => unreachable!("ResourceId {} did not represent a valid resource", id),
        }
    }

    pub fn get_holder_by_id_unchecked(&self, id: ResourceId) -> Rc<ResourceHolder> {
        let resources = self.resources.borrow();
        // SAFETY: A ResourceId is always created from a valid index. Resources are never removed from the list.
        unsafe { (*resources).get_unchecked(id.0).clone() }
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (ResourceId, Rc<ResourceHolder>)> {
        self.iter().enumerate().map(|(i, r)| (ResourceId(i), r))
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Rc<ResourceHolder>> + 'a {
        // resources is in a RefCell, We need to implement our own iterator to avoid cloning the whole vec
        struct ResourceManagerIter<'a> {
            inner: &'a ResourceManager,
            idx: usize,
        }
        impl<'a> Iterator for ResourceManagerIter<'a> {
            type Item = Rc<ResourceHolder>;
            fn next(&mut self) -> Option<Self::Item> {
                let idx = self.idx;
                self.idx += 1;
                self.inner.resources.borrow().get(idx).cloned()
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining = self.inner.resources.borrow().len().saturating_sub(self.idx);
                (remaining, Some(remaining))
            }
        }

        return ResourceManagerIter {
            inner: self,
            idx: 0,
        };
    }

    #[deprecated(
        note = "Use get_id_by_path + get_by_id instead and cache the ID. This function is O(n)."
    )]
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
        assigned_id: ResourceId,
        dependency_reporter: &DependencyReporter,
        lua: Rc<mlua::Lua>,
        gl: Arc<glow::Context>,
        path: &Path,
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
