use std::rc::Rc;

use crate::lua_env::add_fn_to_table;

pub fn setup_persist_api(lua: &Rc<mlua::Lua>) -> mlua::Result<mlua::Table> {
    let persist_module = lua.create_table()?;

    add_fn_to_table(lua, &persist_module, "onReload", {
        move |lua, (default_value, global_name): (mlua::Value, String)| {
            let g = lua.globals();
            let value = g.raw_get::<mlua::Value>(global_name.clone());
            if let Ok(value) = value
                && !value.is_nil()
            {
                return Ok(value);
            }
            let _ = g.raw_set(global_name, default_value.clone());
            Ok(default_value)
        }
    });

    Ok(persist_module)
}
