use mlua::{FromLua, UserData};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Verbosity {
    Info,  // White
    Warn,  // Yellow
    Error, // Red
}

impl UserData for Verbosity {}
impl FromLua for Verbosity {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Verbosity".to_string(),
                message: Some("Expected Verbosity userdata".to_string()),
            }),
        }
    }
}

#[derive(Debug)]
pub struct ConsoleMessage {
    pub verbosity: Verbosity,
    pub msg: String,
}
