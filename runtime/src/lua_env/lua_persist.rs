use std::{fs::OpenOptions, io::Write, path::PathBuf, rc::Rc};

use mlua::LuaSerdeExt;
use serde_json;

use crate::lua_env::add_fn_to_table;

fn serialize_lua(lua: &mlua::Lua, value: &mlua::Value) -> Box<[u8]> {
    let json_value: Result<serde_json::Value, _> = lua.from_value(value.clone());
    match json_value {
        Ok(json) => serde_json::to_vec(&json)
            .unwrap_or_default()
            .into_boxed_slice(),
        Err(_) => vec![].into_boxed_slice(),
    }
}

fn deserialize_lua(lua: &mlua::Lua, value: Box<[u8]>) -> mlua::Result<mlua::Value> {
    let json_value: serde_json::Value =
        serde_json::from_slice(&value).map_err(|e| mlua::Error::DeserializeError(e.to_string()))?;
    lua.to_value(&json_value)
}

fn get_kv_store_path() -> std::path::PathBuf {
    let exec_path = std::env::current_exe().ok();
    let data_folder = exec_path.and_then(|p| p.parent().map(|p| p.join("data")));
    if let Some(data_folder) = data_folder {
        return data_folder;
    }
    PathBuf::from("data")
}

fn save_data_in_kv_store(key: String, value: Box<[u8]>) {
    let path = get_kv_store_path();
    let path = path.join(format!("{}.bin", key));
    let prefix = path.parent().expect("No parent");
    std::fs::create_dir_all(prefix).expect("Unable to create directory");
    let mut file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(err) => {
            println!("Unable to create file: {}", err);
            return;
        }
    };
    let _ = file.write_all(&value);

    #[cfg(target_os = "emscripten")]
    {
        use emscripten_functions::emscripten::run_script;
        run_script(
            r#"
            FS.syncfs(false, function (err) {
                if (err) console.error('Failed to persist data to IndexedDB:', err);
            });
        "#,
        );
    }
}

fn load_data_from_kv_store(key: String) -> Option<Box<[u8]>> {
    let path = get_kv_store_path();
    let path = path.join(format!("{}.bin", key));
    std::fs::read(&path).ok().map(|v| v.into_boxed_slice())
}

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

    add_fn_to_table(lua, &persist_module, "onReloadWithProvider", {
        move |lua, (provider, global_name): (mlua::Function, String)| {
            let g = lua.globals();
            let value = g.raw_get::<mlua::Value>(global_name.clone());
            if let Ok(value) = value
                && !value.is_nil()
            {
                return Ok(value);
            }
            let default_value: mlua::Value = provider.call(())?;
            let _ = g.raw_set(global_name, default_value.clone());
            Ok(default_value)
        }
    });

    add_fn_to_table(lua, &persist_module, "load", {
        move |lua, (key,): (String,)| {
            let data = load_data_from_kv_store(key);
            let Some(data) = data else {
                return Ok(mlua::Nil);
            };
            deserialize_lua(lua, data)
        }
    });

    add_fn_to_table(lua, &persist_module, "save", {
        move |lua, (key, value): (String, mlua::Value)| {
            let value = serialize_lua(lua, &value);
            save_data_in_kv_store(key, value);
            Ok(())
        }
    });

    Ok(persist_module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load() {
        let key = "test_key".to_string();
        let data = vec![1, 2, 3, 4, 5].into_boxed_slice();

        save_data_in_kv_store(key.clone(), data.clone());
        let loaded = load_data_from_kv_store(key);

        assert_eq!(Some(data), loaded);
    }

    #[test]
    fn serialize_lua_and_back() {
        let lua = mlua::Lua::new();
        let value = lua
            .to_value(&"test")
            .expect("Unable to convert value to lua");
        let serialized = serialize_lua(&lua, &value);
        let deserialized = deserialize_lua(&lua, serialized).expect("Unable to deserialize value");
        assert_eq!(value, deserialized);
    }
}
