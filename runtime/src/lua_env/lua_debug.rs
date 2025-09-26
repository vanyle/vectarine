use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    console::{ConsoleMessage, Verbosity},
    lua_env::{add_fn_to_table, get_internals, stringify_lua_value},
};

pub fn setup_debug_api(
    lua: &Rc<mlua::Lua>,
    messages: &Rc<RefCell<VecDeque<ConsoleMessage>>>,
    frame_messages: &Rc<RefCell<Vec<ConsoleMessage>>>,
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

    // Put a print function inside an internal table so that it can be called from anywhere in Rust.
    let internals = get_internals(lua);
    internals.raw_set(
        "print",
        lua.create_function({
            let messages = messages.clone();
            move |_, (msg, verbosity): (String, Verbosity)| {
                messages
                    .borrow_mut()
                    .push_front(ConsoleMessage { msg, verbosity });
                Ok(())
            }
        })
        .unwrap(),
    )?;

    Ok(debug_module)
}
