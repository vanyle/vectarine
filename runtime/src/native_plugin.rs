pub mod native_plugin_impl;

use std::rc::Rc;

use vectarine_plugin_sdk::plugininterface::{EditorPluginInterface, PluginInterface};

use crate::game_resource::ResourceManager;

#[cfg(target_os = "emscripten")]
use super::native_plugin::native_plugin_impl::emscripten as imp;

#[cfg(not(target_os = "emscripten"))]
use super::native_plugin::native_plugin_impl::desktop as imp;

pub enum Platform {
    Linux,
    Windows,
    MacOS,
    WebEmscripten,
}

pub struct NativePlugin {
    native_handle: imp::NativePlugin,
    name: String,     // as specified in the .vecta file of the game
    location: String, // the path/url used to access the plugin data. Usually, it is the name concatenated with something.
}

impl NativePlugin {
    /// Load a native vectarine plugin from a path.
    pub fn load(name: &str, location: &str) -> vectarine_plugin_sdk::anyhow::Result<Self> {
        let native_handle = unsafe { imp::NativePlugin::load(location) }?;
        Ok(Self {
            native_handle,
            name: name.to_string(),
            location: location.to_string(),
        })
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_location(&self) -> String {
        self.location.clone()
    }

    pub fn call_init_hook(&self, plugin_interface: PluginInterface) {
        self.native_handle.call_init_hook(plugin_interface)
    }

    pub fn call_release_hook(&self, plugin_interface: PluginInterface) {
        self.native_handle.call_release_hook(plugin_interface)
    }

    pub fn call_pre_lua_hook(&self, plugin_interface: PluginInterface) {
        self.native_handle.call_pre_lua_hook(plugin_interface)
    }

    pub fn call_post_lua_hook(&self, plugin_interface: PluginInterface) {
        self.native_handle.call_post_lua_hook(plugin_interface)
    }

    pub fn call_draw_debug_menu_hook(&self, plugin_interface: EditorPluginInterface) -> bool {
        self.native_handle
            .call_draw_debug_menu_hook(plugin_interface)
    }
}

pub struct PluginEnvironment {
    // Rc because in an editor environment, the loaded_plugin are a subset of the available plugins, so we only store a reference to them.
    pub loaded_plugins: Vec<Rc<NativePlugin>>,
}

impl PluginEnvironment {
    /// Are plugins resources? Great question! No. But we still need a resource_manager to resolve their path.
    pub fn load_plugins(plugin_names: &[String], resource_manager: &ResourceManager) -> Self {
        // TODO: load plugins from a directory in a cross-platform way
        let suffix = get_dynamic_lib_suffix();
        let native_plugins = plugin_names
            .iter()
            .flat_map(|name| {
                let full_name = format!("{}.{}", name, suffix);
                // We look at the plugin at multiple locations before giving up
                let plugin_path = resource_manager
                    .get_resource_path()
                    .join("plugins")
                    .join(&full_name);
                if !plugin_path.exists() {
                    // exists is not available for the web...
                    return None;
                }
                let plugin = match NativePlugin::load(name, plugin_path.to_string_lossy().as_ref())
                {
                    Ok(plugin) => plugin,
                    Err(e) => {
                        println!("Failed to load plugin {}: {}", full_name, e);
                        return None;
                    }
                };
                Some(Rc::new(plugin))
            })
            .collect::<Vec<_>>();

        Self {
            loaded_plugins: native_plugins,
        }
    }

    pub fn get_plugins(&self) -> impl Iterator<Item = &Rc<NativePlugin>> {
        self.loaded_plugins.iter()
    }

    /// Call the initialization hook of all the loaded plugins
    pub fn init(&self, plugin_interface: PluginInterface) {
        for plugin in &self.loaded_plugins {
            plugin.call_init_hook(plugin_interface); // might trigger a crash I guess?
        }
    }

    pub fn pre_lua_hook(&self, plugin_interface: PluginInterface) {
        for plugin in &self.loaded_plugins {
            plugin.call_pre_lua_hook(plugin_interface);
        }
    }

    pub fn post_lua_hook(&self, plugin_interface: PluginInterface) {
        for plugin in &self.loaded_plugins {
            plugin.call_post_lua_hook(plugin_interface);
        }
    }

    /// Call the release hook of all the loaded plugins
    pub fn release_hook(&self, plugin_interface: PluginInterface) {
        for plugin in &self.loaded_plugins {
            plugin.call_release_hook(plugin_interface);
        }
    }
}

pub static DYNAMIC_LIB_SUFFIXES: [&str; 4] = ["so", "dll", "dylib", "wasm"];

pub fn get_dynamic_lib_suffix() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "so"
    }
    #[cfg(target_os = "windows")]
    {
        "dll"
    }
    #[cfg(target_os = "macos")]
    {
        "dylib"
    }
    #[cfg(target_os = "emscripten")]
    {
        "wasm"
    }
}
