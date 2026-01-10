pub mod native_plugin_impl;

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

/// Plugin interface is an object that is accessed by the plugin to interact with the runtime
struct PluginInterface {
    // ...
}

impl NativePlugin {
    pub fn load(_path: &str) -> Self {
        todo!();
    }
    pub fn get_filename(&self) -> String {
        self.filename.clone()
    }
}

#[link(name = "libeditor_plugin_template.dylib", kind = "dylib")]
unsafe extern "C" {
    fn add(left: usize, right: usize) -> usize;
}
