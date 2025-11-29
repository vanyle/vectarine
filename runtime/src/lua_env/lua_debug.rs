use std::{cell::RefCell, rc::Rc};

use crate::console::{print_frame, print_info};
use crate::lua_env::{add_fn_to_table, stringify_lua_value};

use crate::metrics::MetricsHolder;

pub fn setup_debug_api(
    lua: &Rc<mlua::Lua>,
    metrics: &Rc<RefCell<MetricsHolder>>,
) -> mlua::Result<mlua::Table> {
    let debug_module = lua.create_table()?;

    add_fn_to_table(lua, &debug_module, "fprint", {
        move |_, args: mlua::Variadic<mlua::Value>| {
            let msg = args
                .iter()
                .map(stringify_lua_value)
                .collect::<Vec<_>>()
                .join("");
            print_frame(msg);
            Ok(())
        }
    });

    add_fn_to_table(lua, &debug_module, "print", {
        move |_, args: mlua::Variadic<mlua::Value>| {
            let msg = args
                .iter()
                .map(stringify_lua_value)
                .collect::<Vec<_>>()
                .join("");
            print_info(msg);
            Ok(())
        }
    });

    add_fn_to_table(lua, &debug_module, "timed", {
        let metrics = metrics.clone();
        move |_, (name, callback): (String, mlua::Function)| {
            let start = std::time::Instant::now();
            callback.call::<()>(())?;
            let elapsed = start.elapsed();
            metrics.borrow_mut().record_duration_metric(&name, elapsed);
            Ok(())
        }
    });

    Ok(debug_module)
}
