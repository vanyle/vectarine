//! Plugin interface defines the Plugin Interface, a sort of SDK for plugins to interact with the runtime and the editor.

/// The plugin interface object.
///
/// It is used for plugins to interact with the runtime.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginInterface<'a> {
    // The Lua struct is not repr(C), so good luck not using Rust!
    // We could add more fields for C friendliness?
    pub lua: &'a mlua::Lua,
}

impl<'a> PluginInterface<'a> {
    pub fn new(lua: &'a mlua::Lua) -> Self {
        Self { lua }
    }
}

/// The editor plugin interface object.
///
/// Provided when the editor wants your plugin to draw a debug menu.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EditorPluginInterface<'a> {
    pub plugin_interface: PluginInterface<'a>,
    pub gui_context: &'a egui::Context,
}
