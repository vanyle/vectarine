pub mod native_plugin_impl;

use std::rc::Rc;

use vectarine_plugin_sdk::plugininterface::PluginInterface;

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
    name: String, // as specified in the .vecta file
}

impl NativePlugin {
    pub fn load(name: &str) -> vectarine_plugin_sdk::anyhow::Result<Self> {
        let native_handle = unsafe { imp::NativePlugin::load(name) }?;
        Ok(Self {
            native_handle,
            name: name.to_string(),
        })
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn call_init_hook(&self, plugin_interface: PluginInterface) {
        self.native_handle.call_init_hook(plugin_interface);
    }

    pub fn call_release_hook(&self, plugin_interface: PluginInterface) {
        self.native_handle.call_release_hook(plugin_interface);
    }

    pub fn call_pre_lua_hook(&self, plugin_interface: PluginInterface) {
        self.native_handle.call_pre_lua_hook(plugin_interface);
    }

    pub fn call_post_lua_hook(&self, plugin_interface: PluginInterface) {
        self.native_handle.call_post_lua_hook(plugin_interface);
    }
}

pub struct PluginEnvironment {
    // Rc because in an editor environment, the loaded_plugin are a subset of the available plugins, so we only store a reference to them.
    pub loaded_plugins: Vec<Rc<NativePlugin>>,
}

impl PluginEnvironment {
    pub fn load_plugins(plugin_names: &[String]) -> Self {
        // TODO: load plugins from a directory in a cross-platform way
        let suffix = get_dynlib_suffix();
        let native_plugins = plugin_names
            .iter()
            .flat_map(|name| {
                let full_name = format!("{}{}", name, suffix);

                // We look at the plugin at multiple locations before giving up
                let plugin = match NativePlugin::load(&full_name) {
                    Ok(plugin) => plugin,
                    Err(e) => {
                        println!("Failed to load plugin {}, {}", full_name, e);
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

    pub fn init(&self, plugin_interface: PluginInterface) {
        for plugin in &self.loaded_plugins {
            plugin.call_init_hook(plugin_interface);
        }
    }
}

fn get_dynlib_suffix() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        ".so"
    }
    #[cfg(target_os = "windows")]
    {
        ".dll"
    }
    #[cfg(target_os = "macos")]
    {
        ".dylib"
    }
    #[cfg(target_os = "emscripten")]
    {
        ".wasm"
    }
}
