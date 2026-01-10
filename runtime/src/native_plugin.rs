pub mod native_plugin_impl;

use crate::native_plugin::plugininterface::PluginInterface;

#[cfg(target_os = "emscripten")]
use super::native_plugin::native_plugin_impl::emscripten as imp;

#[cfg(not(target_os = "emscripten"))]
use super::native_plugin::native_plugin_impl::desktop as imp;

pub mod plugininterface;

pub enum Platform {
    Linux,
    Windows,
    MacOS,
    WebEmscripten,
}

pub struct NativePlugin {
    native_handle: imp::NativePlugin,

    filename: String,

    /// A piece of code from the plugin to run every frame before Lua
    pre_lua_hook: Option<Box<dyn FnMut()>>,

    /// A piece of code from the plugin to run every frame after Lua
    post_lua_hook: Option<Box<dyn FnMut()>>,

    /// A piece of code from the plugin to run when the game is loaded
    /// You should add your Lua functions to the Lua environment here
    init_hook: Option<Box<dyn FnMut()>>,

    /// A piece of code from the plugin to run when the game is unloaded
    /// If you need to do memory management, do it here.
    release_hook: Option<Box<dyn FnMut()>>,
}

impl NativePlugin {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let native_handle = unsafe { imp::NativePlugin::load(path) }?;
        Ok(Self {
            native_handle,
            filename: path.to_string(),
            pre_lua_hook: None,
            post_lua_hook: None,
            init_hook: None,
            release_hook: None,
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

    pub fn call_pre_lua_hook(&self) {
        self.native_handle.call_pre_lua_hook();
    }

    pub fn call_post_lua_hook(&self) {
        self.native_handle.call_post_lua_hook();
    }
}
