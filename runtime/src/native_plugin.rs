pub mod native_plugin_impl;

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
    filename: String,
}

impl NativePlugin {
    pub fn load(path: &str) -> vectarine_plugin_sdk::anyhow::Result<Self> {
        let native_handle = unsafe { imp::NativePlugin::load(path) }?;
        Ok(Self {
            native_handle,
            filename: path.to_string(),
        })
    }
    pub fn get_filename(&self) -> String {
        self.filename.clone()
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
