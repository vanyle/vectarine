//! Plugin interface defines the Plugin Interface, a sort of SDK for plugins to interact with the runtime and the editor.

/// The plugin interface object.
#[repr(C)]
pub struct PluginInterface<'lua> {
    pub lua: &'lua mlua::Lua,
    pub number: u32,
}

impl<'lua> PluginInterface<'lua> {
    pub fn new(lua: &'lua mlua::Lua, number: u32) -> Self {
        Self { lua, number }
    }
}

#[repr(C)]
pub struct EditorPluginInterface {
    // ...
}
