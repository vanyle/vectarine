use vectarine_plugin_sdk::{
    egui,
    mlua::ffi,
    plugininterface::{EditorPluginInterface, PluginInterface},
};

unsafe extern "C-unwind" fn square_number(state: *mut ffi::lua_State) -> i32 {
    unsafe {
        let n = ffi::luaL_checknumber(state, 1);
        ffi::lua_pushnumber(state, n * n);
        1
    }
}

/// The init_hook is called when the game is loaded. You can use it to register custom lua functions, variables, etc...
#[unsafe(no_mangle)]
pub extern "C" fn init_hook(plugin_interface: PluginInterface) {
    // This function is called once when the game is loaded.
    // In this example, there is a print to show that the plugin is loaded, but in production, you don't want
    // to print here as you are just adding noise.
    println!("The Vectarine Plugin Template was loaded!");

    // While plugins have full control over the Lua state, changing it is bad practice as you can break other plugins or the game
    // in unpredictable ways. You should register modules to extend the Luau API (see below)
    //
    // let _ = plugin_interface
    //     .lua
    //     .globals()
    //     .set("lua_plugin_template_version", "0.1.0");

    // we need to define a table that gets required using require("@vectarine/plugin_template")
    let lua = plugin_interface.lua;
    let value = lua.create_table().expect("Failed to create Lua table");

    // We are reimplementing the content of plugin.luau in Rust.
    // Having a version field is useful so that when developing you remember to run `uv run bundle.py --install` to update the native code.
    let _ = value.set("VERSION", 2);
    let _ = value.set("NAME", "Plugin Template");

    unsafe {
        // Due to how mlua works, we need to use "create_c_function" instead of "create_function".
        let square_fn = lua
            .create_c_function(square_number)
            .expect("Failed to create Lua c-function");

        let _ = value.set("square", square_fn);
    }

    // Actually register the module. The module name here should match the name you put in the manifest.
    let _ = lua.register_module("@vectarine/plugin_template", value);
}

/// The release_hook is called when the game is unloaded. You can use it to free resources if needed.
/// You don't need to define it if you don't need it. If it is not defined, it simply won't be called.
#[unsafe(no_mangle)]
pub extern "C" fn release_hook(_plugin_interface: PluginInterface) {
    // ...
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
