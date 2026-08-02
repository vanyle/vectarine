use std::{
    cell::RefCell,
    collections::HashSet,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::Arc,
};

use vectarine_plugin_sdk::mlua::IntoLua;
use vectarine_plugin_sdk::serde::{Deserialize, Serialize};
use vectarine_plugin_sdk::{egui::ahash::HashMap, glow};

use crate::{
    game_resource::script_resource::ScriptResource,
    io::{dummyfs::DummyFileSystem, fs::ReadOnlyFileSystem},
    lua_env::{LuaHandle, lua_event::EventType},
};

pub mod audio_resource;
pub mod font_resource;
pub mod image_resource;
pub mod script_resource;
pub mod shader_resource;
pub mod text_resource;
pub mod tile_resource;

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
#[serde(crate = "vectarine_plugin_sdk::serde")]
pub struct ResourceId(usize);

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceId({})", self.0)
    }
}

impl ResourceId {
    pub fn get_id(&self) -> usize {
        self.0
    }
}

pub struct ResourceHolder {
    resource: Rc<dyn Resource>,
    status: RefCell<Status>,

    name: String,
    /// The path to the resources after its canonicalization. There is a bijection between paths and resources. This path is relative to the game project folder.
    path: PathBuf,
    /// A list of ids of other resources that this resource needs to work
    dependencies: RefCell<HashSet<ResourceId>>,
    /// A list of ids of other resources that depend on this resource
    dependent: RefCell<HashSet<ResourceId>>,

    /// The path of the resource that caused this resource to be loaded. Is used to resolve relative paths.
    loading_cause_resource_path: Option<PathBuf>,
}

impl ResourceHolder {
    /// Request the resource to be reloaded.
    fn reload(
        self: Rc<Self>,
        file_system: &dyn ReadOnlyFileSystem,
        assigned_id: ResourceId,
        resource_manager: Rc<ResourceManager>,
        gl: Arc<glow::Context>,
        lua: Rc<LuaHandle>,
        resource_event: EventType,
    ) {
        if self.is_loading() {
            return;
        }

        // Clean ourselves from dependent array of others:
        for dep_id in self.dependencies.borrow().iter() {
            let dep = resource_manager.get_holder_by_id(*dep_id);
            dep.dependent.borrow_mut().remove(&assigned_id);
        }
        self.dependencies.borrow_mut().clear();

        let dr = DependencyReporter {
            resource_manager: Rc::downgrade(&resource_manager),
        };

        self.status.replace(Status::Loading);

        let abs_path = match validate_and_canonicalize_resource_path(
            &self.path,
            &resource_manager.base_path,
            self.loading_cause_resource_path.as_deref(),
        ) {
            Err(cause) => {
                self.status.replace(Status::Error(cause));
                return;
            }
            Ok(abs_path) => abs_path,
        };

        // We pass data to the resource into the closure.
        // As this data needs to be kept alive, every piece of state pass inside needs Rc or Arc.
        file_system.read_file(
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
                    &lua,
                    gl.clone(),
                    &self.path,
                    data.into_boxed_slice(),
                );
                self.status.replace(resulting_status);
                let _ = resource_event.trigger(
                    assigned_id
                        .get_id()
                        .into_lua(&lua.lua)
                        .expect("Failed to convert usize to Lua"),
                );
            }),
        );
    }

    /// For resources that are loaded in an async manner, you can use this to mark them as ready if you have access to the resource holder.
    pub fn mark_as_ready(&self) {
        self.status.replace(Status::Loaded);
    }

    /// For resources that are loaded in an async manner, you can use this to mark them as error if you have access to the resource holder.
    pub fn mark_as_error(&self, error_message: String) {
        self.status.replace(Status::Error(error_message));
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

    pub fn draw_debug_gui(
        &self,
        painter: &mut vectarine_plugin_sdk::egui_glow::Painter,
        ui: &mut vectarine_plugin_sdk::egui::Ui,
    ) {
        self.resource.draw_debug_gui(painter, ui);
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

pub struct ResourceManager {
    file_system: Box<dyn ReadOnlyFileSystem>,
    resources: RefCell<Vec<Rc<ResourceHolder>>>,
    // Optimization to find resource ids in O(1)
    path_to_id: RefCell<HashMap<PathBuf, ResourceId>>,
    base_path: PathBuf,
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

#[derive(Debug, Clone)]
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

    /// Obtain a ResourceId to a resource you depend on. If the resource is not loaded yet, return None.
    /// This function runs in O(N) currently.
    /// In that case, you should declare the dependency and return Unloaded to wait for the resource to be loaded.
    pub fn obtain_resource_id(&self, path: &Path) -> Option<ResourceId> {
        let resource_manager = self.resource_manager.upgrade()?;
        resource_manager.get_id_by_path(path)
    }

    pub fn obtain_resource<T: Resource + 'static>(
        &self,
        resource_id: &ResourceId,
    ) -> Result<Rc<T>, String> {
        let resource_manager = self
            .resource_manager
            .upgrade()
            .ok_or_else(|| "Failed to upgrade ResourceManager".to_string())?;
        resource_manager.get_by_id::<T>(*resource_id)
    }

    /// For resources that are loaded in an async manner, you can use this to mark them as ready if you have access to the resource holder.
    pub fn mark_as_ready(&self, resource_id: ResourceId) {
        let resource_manager = self
            .resource_manager
            .upgrade()
            .expect("Failed to upgrade ResourceManager");
        let holder = resource_manager.get_holder_by_id(resource_id);
        holder.mark_as_ready();
    }

    /// For resources that are loaded in an async manner, you can use this to mark them as error if you have access to the resource holder.
    pub fn mark_as_error(&self, resource_id: ResourceId, error_message: String) {
        let resource_manager = self
            .resource_manager
            .upgrade()
            .expect("Failed to upgrade ResourceManager");
        let holder = resource_manager.get_holder_by_id(resource_id);
        holder.mark_as_error(error_message);
    }
}

impl ResourceManager {
    pub fn new(file_system: Box<dyn ReadOnlyFileSystem>, base_path: &Path) -> Self {
        Self {
            resources: RefCell::new(Vec::new()),
            base_path: base_path.to_path_buf(),
            file_system,
            path_to_id: RefCell::new(HashMap::default()),
        }
    }

    pub fn file_system(&self) -> &dyn ReadOnlyFileSystem {
        &*self.file_system
    }

    /// Used when function seem to want a resource manager, but don't actually need it.
    /// Contains no resources and cannot load them. All resources will get an error status.
    pub fn dummy_manager() -> Self {
        Self {
            resources: RefCell::new(Vec::new()),
            base_path: PathBuf::new(),
            file_system: Box::new(DummyFileSystem {}),
            path_to_id: RefCell::new(HashMap::default()),
        }
    }

    /// Create a new resource from a file and schedule it for loading.
    /// If the resource already exists at that path, do nothing.
    /// Return the id of the resource.
    pub fn schedule_load_resource<T: Resource + 'static>(
        &self,
        path: &Path,
        loading_cause_path: Option<&Path>,
    ) -> ResourceId {
        self.schedule_load_resource_with_builder::<T, _>(path, loading_cause_path, T::default)
    }

    /// Create a new resource from a file and schedule it for loading.
    /// If the resource already exists at that path, do nothing.
    /// Return the id of the resource.
    /// The builder function is called to create the unloaded resource instance.
    /// We try to validate the path is possible. If we cannot, we use the path as-is.
    pub fn schedule_load_resource_with_builder<T: Resource + 'static, F: FnOnce() -> T>(
        &self,
        path: &Path,
        loading_cause_path: Option<&Path>,
        builder: F,
    ) -> ResourceId {
        let standard_path = resolve_dot_relative_paths(path, loading_cause_path);
        let canonical_path = get_canonical_absolute_path(&self.base_path, &standard_path);

        if let Some(&id) = self.path_to_id.borrow().get(&canonical_path) {
            return id;
        }

        let id = ResourceId(self.resources.borrow().len());
        let resource = Rc::new(builder());
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        self.resources.borrow_mut().push(Rc::new(ResourceHolder {
            status: RefCell::new(Status::Unloaded),
            path: standard_path,
            name,
            dependencies: RefCell::new(HashSet::new()),
            dependent: RefCell::new(HashSet::new()),
            loading_cause_resource_path: loading_cause_path.map(|p| p.to_path_buf()),
            resource,
        }));

        self.path_to_id.borrow_mut().insert(canonical_path, id);

        id
    }

    pub fn schedule_load_script_resource(
        &self,
        path: &Path,
        loading_cause_path: Option<&Path>,
        target_table: vectarine_plugin_sdk::mlua::Table,
    ) -> (ResourceId, vectarine_plugin_sdk::mlua::Table) {
        if let Some(id) = self.get_id_by_path(path) {
            let script_resource = self.get_by_id::<ScriptResource>(id);
            let Ok(script_resource) = script_resource else {
                // The resource type changed. This is rare, but it can happen.
                return (id, target_table);
            };
            let exports = script_resource.get_exports();
            let Some(exports) = exports else {
                // The script does not have an export table. This means it was created without one. This is happens
                // when creating a script using schedule_load_resource instead of schedule_load_script_resource.
                // This cannot happen because of Lua.
                return (id, target_table);
            };
            // We return a reference to the exports of the script which is dynamically updated when reloading.
            return (id, exports.clone());
        }
        let rid = self.schedule_load_resource_with_builder(path, loading_cause_path, || {
            ScriptResource::make_with_target_table(target_table.clone())
        });
        (rid, target_table)
    }

    /// Create a new resource from a file and start loading it immediately.
    /// If the resource already exists at that path, do nothing.
    /// Return the id of the resource.
    pub fn load_resource<T: Resource + 'static>(
        self: &Rc<Self>,
        path: &Path,
        loading_cause_path: Option<&Path>,
        gl: Arc<glow::Context>,
        lua: Rc<LuaHandle>,
        loaded_event: EventType,
    ) -> ResourceId {
        if let Some(id) = self.get_id_by_path(path) {
            return id;
        }
        let id = self.schedule_load_resource::<T>(path, loading_cause_path);
        self.reload(id, gl, lua, loaded_event);
        id
    }

    /// Declare that the resource with id `resource_id` depends on the resource at `path`.
    fn declare_dependency<T: Resource + 'static>(
        self: &Rc<Self>,
        resource_id: ResourceId,
        path: &Path,
    ) {
        let resource: Rc<ResourceHolder> = {
            let resources = self.resources.borrow();
            let Some(resource) = resources.get(resource_id.0) else {
                unreachable!("Incorrect resource id {}", resource_id.0);
            };
            resource.clone()
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
        let loading_cause_path = Some(resource.path.as_path()); // Self caused the resource at `path` to be loaded.
        self.schedule_load_resource::<T>(path, loading_cause_path);
    }

    pub fn reload(
        self: &Rc<Self>,
        id: ResourceId,
        gl: Arc<glow::Context>,
        lua: Rc<LuaHandle>,
        loaded_event: EventType,
    ) {
        let resource = self.get_holder_by_id(id);
        resource.reload(
            self.file_system.as_ref(),
            id,
            self.clone(),
            gl,
            lua,
            loaded_event,
        );
    }

    pub fn get_id_by_path(&self, path: &Path) -> Option<ResourceId> {
        let to_match = get_canonical_absolute_path(&self.base_path, path);
        self.path_to_id.borrow().get(&to_match).copied()
    }

    pub fn get_by_id<T: Resource + 'static>(&self, id: ResourceId) -> Result<Rc<T>, String> {
        let resource = self.get_holder_by_id(id);
        if !resource.is_loaded() {
            return Err("Resource not available yet".into());
        }
        resource.get_underlying_resource::<T>()
    }

    /// Get a resource, even if it is not ready yet.
    pub fn get_loading_resource_by_id<T: Resource + 'static>(
        &self,
        id: ResourceId,
    ) -> Result<Rc<T>, String> {
        let resource = self.get_holder_by_id(id);
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

    pub fn iter(&self) -> impl Iterator<Item = Rc<ResourceHolder>> + '_ {
        // resources is in a RefCell, We need to implement our own iterator to avoid cloning the whole vec
        struct ResourceManagerIter<'a> {
            inner: &'a ResourceManager,
            idx: usize,
        }
        impl Iterator for ResourceManagerIter<'_> {
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

        ResourceManagerIter {
            inner: self,
            idx: 0,
        }
    }

    #[deprecated(
        note = "Use get_id_by_path + get_by_id instead and cache the ID. This function is O(n)."
    )]
    pub fn get_by_path(&self, path: &Path) -> Option<Rc<dyn Resource>> {
        let to_match = get_absolute_path(&self.base_path, path);

        for res in self.resources.borrow().iter() {
            let p1 = get_absolute_path(&self.base_path, &res.path);
            if to_match == p1 {
                return Some(res.resource.clone());
            }
        }
        None
    }

    pub fn get_absolute_path(&self, resource_path: &Path) -> String {
        get_absolute_path(&self.base_path, resource_path)
    }
    pub fn get_resource_path(&self) -> PathBuf {
        self.base_path.clone()
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
        lua: &Rc<LuaHandle>,
        gl: Arc<glow::Context>,
        path: &Path,
        data: Box<[u8]>,
    ) -> Status;

    /// Draw an interface with information about the resource.
    fn draw_debug_gui(
        &self,
        painter: &mut vectarine_plugin_sdk::egui_glow::Painter,
        ui: &mut vectarine_plugin_sdk::egui::Ui,
    );

    /// A human-friendly name for this type of Resource.
    /// This is usually the name of the struct implementing the trait.
    fn get_type_name(&self) -> &'static str;

    /// Create an empty instance of a resource
    fn default() -> Self
    where
        Self: Sized;
}

pub fn get_absolute_path(current_base_path: &Path, resource_path: &Path) -> String {
    // canonicalize cannot be called here. we are in the runtime, there is potentially no true filesystem!
    let abs_path = current_base_path.join(resource_path);

    // Absolutize by removing . and .. components. We need to make the path unique.
    let components = abs_path.components().collect::<Vec<_>>();
    let mut stack = Vec::new();
    for component in components {
        match component {
            std::path::Component::ParentDir => {
                stack.pop();
            }
            std::path::Component::CurDir => {}
            _ => {
                stack.push(component);
            }
        }
    }
    let abs_path = stack.iter().collect::<PathBuf>();
    abs_path.to_string_lossy().replace("\\", "/")
}
pub fn get_canonical_absolute_path(current_base_path: &Path, resource_path: &Path) -> PathBuf {
    current_base_path
        .join(resource_path)
        .canonicalize()
        .unwrap_or_else(|_| current_base_path.join(resource_path))
}

pub fn has_no_uppercase(s: &str) -> bool {
    s.chars().all(|c| !c.is_ascii_uppercase())
}

/// If a path is of the form "./filename.extension", it is relative to the file loading the resource. In this
/// case, we resolve the path provided
pub fn resolve_dot_relative_paths(
    maybe_dot_relative_path: &Path,
    base_path: Option<&Path>,
) -> PathBuf {
    let components = maybe_dot_relative_path.components().collect::<Vec<_>>();
    if components.is_empty() {
        return maybe_dot_relative_path.to_path_buf();
    }
    let first_path = components[0];

    if first_path == std::path::Component::CurDir
        && let Some(base_path) = base_path
    {
        // The path is relative to the loading_cause_path.
        return PathBuf::from(get_absolute_path(
            base_path.parent().expect("Loading cause path has a parent"),
            maybe_dot_relative_path,
        ));
    }
    maybe_dot_relative_path.to_path_buf()
}

/// Depending on the platform, paths can be interpreted in different ways (case-sensitivity, relative vs absolute, etc.)
/// To avoid ambiguity, we ban some types of paths and product and error describing why that path is not valid.
pub fn validate_and_canonicalize_resource_path(
    resource_path: &Path,
    game_project_path: &Path,
    maybe_loading_cause_path: Option<&Path>,
) -> Result<String, String> {
    if !has_no_uppercase(&resource_path.to_string_lossy()) {
        return Err("Resource paths must be lowercase to be cross-platform!".to_string());
    }

    // All paths must be of the form:
    // - "foldername/filename.extension" (in that case, the path is given from the root of the game project folder)
    // - "./filename.extension" (in that case, the path is relative to the file loading the resource)

    let resolved_path = resolve_dot_relative_paths(resource_path, maybe_loading_cause_path);
    let abs_path = get_absolute_path(game_project_path, resolved_path.as_path());
    Ok(abs_path)
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
