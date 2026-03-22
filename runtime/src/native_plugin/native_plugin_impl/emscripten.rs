use vectarine_plugin_sdk::plugininterface::{EditorPluginInterface, PluginInterface};

pub(crate) struct NativePlugin {}

impl NativePlugin {
    /// # Safety
    ///
    /// This function is unsafe because it loads a native module. On some platforms, native module can run code when loaded.
    /// Such a module can run any code and is inherently unsafe.
    pub unsafe fn load(_path: &str) -> vectarine_plugin_sdk::anyhow::Result<Self> {
        Err(vectarine_plugin_sdk::anyhow::Error::msg(
            "Native plugins are not supported on WebAssembly yet",
        ))
    }

    pub fn call_init_hook(&self, _plugin_interface: PluginInterface) {}

    pub fn call_release_hook(&self, _plugin_interface: PluginInterface) {}

    pub fn call_pre_lua_hook(&self, _plugin_interface: PluginInterface) {}

    pub fn call_post_lua_hook(&self, _plugin_interface: PluginInterface) {}

    pub fn call_draw_debug_menu_hook(&self, _plugin_interface: EditorPluginInterface) -> bool {
        false
    }
}
