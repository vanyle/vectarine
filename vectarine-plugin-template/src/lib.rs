use vectarine_plugin_sdk::{
    egui,
    plugininterface::{EditorPluginInterface, PluginInterface},
};

/// The init_hook is called when the game is loaded. You can use it to register custom lua functions, variables, etc...
#[unsafe(no_mangle)]
pub extern "C" fn init_hook(plugin_interface: PluginInterface) {
    println!("I was loaded!");
    let _ = plugin_interface
        .lua
        .globals()
        .set("lua_plugin_template_version", "0.1.0");
    println!("I changed.... a global value!");
}

/// The release_hook is called when the game is unloaded. You can use it to free resources if needed.
/// You don't need to define it if you don't need it. If it is not defined, it simply won't be called.
#[unsafe(no_mangle)]
pub extern "C" fn release_hook(_plugin_interface: PluginInterface) {
    println!("I was unloaded!");
}

/// The pre_lua_hook is called every frame, before the lua script is executed.
#[unsafe(no_mangle)]
pub extern "C" fn pre_lua_hook(_plugin_interface: PluginInterface) {
    // ...
}

/// The post_lua_hook is called every frame, after the lua script is executed. You can use it to draw overlays.
#[unsafe(no_mangle)]
pub extern "C" fn post_lua_hook(_plugin_interface: PluginInterface) {
    // ...
}

/// The draw_debug_menu_hook is called only in the editor when the debug menu of your extension needs to be drawn.
/// You can use it to add a custom editor window to your plugin.
/// Return true if you want to keep drawing the debug menu and false to close it.
#[unsafe(no_mangle)]
pub extern "C" fn draw_debug_menu_hook(plugin_interface: EditorPluginInterface) -> bool {
    let mut should_stay_open = true;
    egui::Window::new("My Plugin Window").show(plugin_interface.gui_context, |ui| {
        ui.label("Hello from my plugin!");
        if ui.button("Close this menu").clicked() {
            // we return false to close the menu
            should_stay_open = false;
        }
    });
    should_stay_open
}
