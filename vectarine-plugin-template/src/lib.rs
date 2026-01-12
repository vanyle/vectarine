use vectarine_plugin_sdk::plugininterface::PluginInterface;

#[unsafe(no_mangle)]
pub extern "C" fn init_hook(plugin_interface: PluginInterface) {
    println!("I was loaded!");
    let _ = plugin_interface
        .lua
        .globals()
        .set("lua_plugin_template_version", "0.1.0");
    println!("I changed.... a global value!");
}

// We need an SDK module and to have it as dependency for our DLLs.

#[unsafe(no_mangle)]
pub extern "C" fn release_hook(_plugin_interface: PluginInterface) {
    println!("I was unloaded!");
}
