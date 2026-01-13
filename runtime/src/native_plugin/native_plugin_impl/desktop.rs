use libloading::{Library, Symbol};

use vectarine_plugin_sdk::{anyhow, plugininterface::PluginInterface};

pub(crate) struct NativePlugin {
    // Note: library NEEDS to be private or else it could be moved to weird ways leading to the symbols
    // pointing to invalid memory.
    #[allow(dead_code)] // needed for the destructor
    library: Library,

    // We require an init_hook. This is to make sure to be able to provide errors for the plugin creator if they forget
    // no_mangle or have another linking issue instead of just loading None values and silently failing.
    init_hook: Symbol<'static, unsafe extern "C" fn(PluginInterface)>,
    release_hook: Option<Symbol<'static, unsafe extern "C" fn(PluginInterface)>>,
    pre_lua_hook: Option<Symbol<'static, unsafe extern "C" fn(PluginInterface)>>,
    post_lua_hook: Option<Symbol<'static, unsafe extern "C" fn(PluginInterface)>>,
}

impl NativePlugin {
    /// # Safety
    ///
    /// This function is unsafe because it loads a native module. On some platforms, native module can run code when loaded.
    /// Such a module can run any code and is inherently unsafe.
    pub unsafe fn load(path: &str) -> vectarine_plugin_sdk::anyhow::Result<Self> {
        let lib = unsafe { Library::new(path) };
        let lib = match lib {
            Ok(lib) => lib,
            Err(err) => {
                return Err(vectarine_plugin_sdk::anyhow::anyhow!(
                    "Failed to load library at {path}: {err}"
                ));
            }
        };
        let init_hook = load_symbol::<unsafe extern "C" fn(PluginInterface)>(&lib, "init_hook")?;
        let release_hook =
            load_symbol::<unsafe extern "C" fn(PluginInterface)>(&lib, "release_hook").ok();
        let pre_lua_hook =
            load_symbol::<unsafe extern "C" fn(PluginInterface)>(&lib, "pre_lua_hook").ok();
        let post_lua_hook =
            load_symbol::<unsafe extern "C" fn(PluginInterface)>(&lib, "post_lua_hook").ok();

        Ok(Self {
            library: lib,
            init_hook,
            release_hook,
            pre_lua_hook,
            post_lua_hook,
            // editor_hooks: None, // this is the runtime.
        })
    }

    pub fn call_init_hook(&self, plugin_interface: PluginInterface) {
        let init_hook = &self.init_hook;
        unsafe { init_hook(plugin_interface) }
    }

    pub fn call_release_hook(&self, plugin_interface: PluginInterface) {
        let release_hook = &self.release_hook;
        if let Some(release_hook) = release_hook {
            unsafe { release_hook(plugin_interface) }
        }
    }

    pub fn call_pre_lua_hook(&self, plugin_interface: PluginInterface) {
        let pre_lua_hook = &self.pre_lua_hook;
        if let Some(pre_lua_hook) = pre_lua_hook {
            unsafe { pre_lua_hook(plugin_interface) }
        }
    }

    pub fn call_post_lua_hook(&self, plugin_interface: PluginInterface) {
        let post_lua_hook = &self.post_lua_hook;
        if let Some(post_lua_hook) = post_lua_hook {
            unsafe { post_lua_hook(plugin_interface) }
        }
    }
}

fn load_symbol<T>(lib: &Library, name: &str) -> anyhow::Result<Symbol<'static, T>> {
    let symbol = unsafe { lib.get::<T>(name) };
    let symbol = match symbol {
        Ok(symbol) => symbol,
        Err(err) => {
            return Err(anyhow::anyhow!(
                "Library does not export symbol '{}': {}",
                name,
                err
            ));
        }
    };
    // We know that symbol, library and the Self struct have the same lifetime, so let's transmute
    // If library is dropped, symbol won't be dropped, but it doesn't matter because they're both in the same struct, so they will be dropped together anyway.
    // The same is true for the other hooks.
    let symbol = unsafe { std::mem::transmute::<Symbol<'_, _>, Symbol<'static, _>>(symbol) };
    Ok(symbol)
}
