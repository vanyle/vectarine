//! Plugin interface defines the Plugin Interface, a sort of SDK for plugins to interact with the runtime and the editor.

/// The plugin interface object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginInterface<'a> {
    pub lua: &'a mlua::Lua,
}

impl<'a> PluginInterface<'a> {
    pub fn new(lua: &'a mlua::Lua) -> Self {
        Self { lua }
    }
}

/// The editor plugin interface object.
/// Provided when the editor wants your plugin to draw a debug menu.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EditorPluginInterface<'a> {
    pub plugin_interface: PluginInterface<'a>,
    pub gui_context: &'a egui::Context,
}
