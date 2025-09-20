use std::{collections::HashMap, rc::Rc};

use crate::lua_env::{add_fn_to_table, get_internals};

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct EventType(u32);

pub struct EventManager {
    pub events: Vec<(String, EventType)>, // hold the name of every event
    pub subscription_tracker: HashMap<u32, u32>, // don't know what type to use for now
}

impl mlua::UserData for EventType {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, _etype: &EventType| Ok("Event".to_string()));
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("dispatch", |lua, _etype, data: mlua::Value| {
            // We can access the outside using lua.globals()
            lua.globals();
            // ...
            Ok("Event".to_string())
        });
    }
}

impl mlua::FromLua for EventType {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Event".to_string(),
                message: Some("expected Event userdata".to_string()),
            }),
        }
    }
}

pub fn setup_event_api(lua: &Rc<mlua::Lua>) -> mlua::Result<mlua::Table> {
    let _internals = get_internals(lua);

    // Unfinished.
    let event_module = lua.create_table()?;
    add_fn_to_table(lua, &event_module, "newEvent", |_, name: String| Ok(name));

    Ok(event_module)
}
