use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    console::{ConsoleMessage, Verbosity},
    lua_env::{add_fn_to_table, stringify_lua_value},
};

use crate::metrics::MetricsHolder;

pub fn setup_debug_api(
    lua: &Rc<mlua::Lua>,
    messages: &Rc<RefCell<VecDeque<ConsoleMessage>>>,
    frame_messages: &Rc<RefCell<Vec<ConsoleMessage>>>,
    metrics: &Rc<RefCell<MetricsHolder>>,
) -> mlua::Result<mlua::Table> {
    let debug_module = lua.create_table()?;

    add_fn_to_table(lua, &debug_module, "fprint", {
        let frame_messages = frame_messages.clone();
        move |_, args: mlua::Variadic<mlua::Value>| {
            let msg = args
                .iter()
                .map(stringify_lua_value)
                .collect::<Vec<_>>()
                .join("");
            frame_messages.borrow_mut().push(ConsoleMessage {
                msg,
                verbosity: Verbosity::Info,
            });
            Ok(())
        }
    });

    add_fn_to_table(lua, &debug_module, "print", {
        let messages = messages.clone();
        move |_, args: mlua::Variadic<mlua::Value>| {
            let msg = args
                .iter()
                .map(stringify_lua_value)
                .collect::<Vec<_>>()
                .join("");
            messages.borrow_mut().push_front(ConsoleMessage {
                msg,
                verbosity: Verbosity::Info,
            });
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
