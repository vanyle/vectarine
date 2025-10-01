use std::{cell::RefCell, collections::VecDeque, path::Path, rc::Rc};

use mlua::ObjectLike;

pub mod lua_canvas;
pub mod lua_coord;
pub mod lua_debug;
pub mod lua_event;
pub mod lua_graphics;
pub mod lua_io;
pub mod lua_resource;
pub mod lua_vec2;

use crate::console::{ConsoleMessage, Verbosity};
use crate::game_resource::ResourceManager;
use crate::graphics::batchdraw::BatchDraw2d;
use crate::io::IoEnvState;

pub struct LuaEnvironment {
    pub lua: Rc<mlua::Lua>,
    pub env_state: Rc<RefCell<IoEnvState>>,

    pub batch: Rc<RefCell<BatchDraw2d>>,

    pub default_events: lua_event::DefaultEvents,

    pub frame_messages: Rc<RefCell<Vec<ConsoleMessage>>>,
    pub messages: Rc<RefCell<VecDeque<ConsoleMessage>>>,

    pub resources: Rc<ResourceManager>,
}

impl LuaEnvironment {
    pub fn new(batch: BatchDraw2d) -> Self {
        let batch = Rc::new(RefCell::new(batch));
        let lua_options = mlua::LuaOptions::default();
        let lua_libs = mlua::StdLib::MATH | mlua::StdLib::TABLE | mlua::StdLib::STRING;

        let lua =
            Rc::new(mlua::Lua::new_with(lua_libs, lua_options).expect("Failed to create Lua"));
        lua.set_compiler(
            mlua::Compiler::new()
                .set_optimization_level(2)
                .set_type_info_level(1),
        );
        let _ = lua.sandbox(false);

        let resources = Rc::new(ResourceManager::default());
        let env_state = Rc::new(RefCell::new(IoEnvState::default()));
        let frame_messages = Rc::new(RefCell::new(Vec::new()));
        let messages = Rc::new(RefCell::new(VecDeque::new()));

        lua.globals()
            .raw_set(UNSAFE_INTERNALS_KEY, lua.create_table().unwrap())
            .unwrap(); // Table used to store unsafe function that we need to access from Rust inside Rust.

        let vec2_module = lua_vec2::setup_vec_api(&lua).unwrap();
        lua.register_module("@vectarine/vec", vec2_module).unwrap();

        let coords_module = lua_coord::setup_coords_api(&lua, &env_state).unwrap();
        lua.register_module("@vectarine/coord", coords_module)
            .unwrap();

        let graphics_module =
            lua_graphics::setup_graphics_api(&lua, &batch, &env_state, &resources).unwrap();
        lua.register_module("@vectarine/graphics", graphics_module)
            .unwrap();

        let canvas_module =
            lua_canvas::setup_canvas_api(&lua, &batch, &env_state, &resources).unwrap();
        lua.register_module("@vectarine/canvas", canvas_module)
            .unwrap();

        let io_module = lua_io::setup_io_api(&lua, &env_state).unwrap();
        lua.register_module("@vectarine/io", io_module).unwrap();

        let debug_module = lua_debug::setup_debug_api(&lua, &messages, &frame_messages).unwrap();
        lua.register_module("@vectarine/debug", debug_module)
            .unwrap();

        let resources_module = lua_resource::setup_resource_api(&lua, &resources).unwrap();
        lua.register_module("@vectarine/resources", resources_module)
            .unwrap();

        let (event_module, default_events) = lua_event::setup_event_api(&lua).unwrap();
        lua.register_module("@vectarine/event", event_module)
            .unwrap();

        let original_require = lua.globals().get::<mlua::Function>("require").unwrap();
        add_global_fn(&lua, "require", move |lua, module_name: String| {
            // We provide a custom require with the following features:
            // - Can require @vectarine/* modules (like @vectarine/vec)
            // - Can require files in the script folder by their names.
            if module_name.starts_with("@vectarine/") {
                return original_require.call(module_name);
            }
            let module = lua.create_table()?;
            module.raw_set("@vectarine/filename", module_name.clone())?;
            Ok(module) // We require an empty table as this is just for the types.
        });

        // Add this to utils module?
        add_global_fn(&lua, "toString", move |_, (arg,): (mlua::Value,)| {
            let string = stringify_lua_value(&arg);
            Ok(string)
        });

        LuaEnvironment {
            lua,
            env_state,
            batch,
            frame_messages,
            default_events,
            messages,
            resources,
        }
    }

    pub fn run_file_and_display_error(&self, file_content: &[u8], file_path: &Path) {
        run_file_and_display_error_from_lua_handle(&self.lua, file_content, file_path, None);
    }

    pub fn print(&self, msg: &str, verbosity: Verbosity) {
        print(&self.lua, verbosity, msg);
    }
}

pub fn add_global_fn<F, A, R>(lua: &Rc<mlua::Lua>, name: &str, func: F)
where
    F: Fn(&mlua::Lua, A) -> mlua::Result<R> + 'static,
    A: mlua::FromLuaMulti,
    R: mlua::IntoLuaMulti,
{
    lua.globals()
        .set(name, lua.create_function(func).unwrap())
        .unwrap()
}

pub fn add_fn_to_table<F, A, R>(lua: &Rc<mlua::Lua>, table: &mlua::Table, name: &str, func: F)
where
    F: Fn(&mlua::Lua, A) -> mlua::Result<R> + 'static,
    A: mlua::FromLuaMulti,
    R: mlua::IntoLuaMulti,
{
    table.set(name, lua.create_function(func).unwrap()).unwrap();
}

/// Run the given Lua file content assuming it is at the given path.
/// If the file returns a table, and a target_table is provided, the table will be merged into the target_table.
pub fn run_file_and_display_error_from_lua_handle(
    lua: &Rc<mlua::Lua>,
    file_content: &[u8],
    file_path: &Path,
    target_table: Option<&mlua::Table>,
) {
    let lua_chunk = lua.load(file_content);
    // Note: We could change the optimization level of the chunk here (for example, inside the runtime)
    let result = lua_chunk
        .set_name("@".to_owned() + file_path.to_str().unwrap())
        .eval::<mlua::Value>();

    match result {
        Err(error) => {
            let error_msg = error.to_string();
            print(lua, Verbosity::Error, &error_msg);
        }
        Ok(value) => {
            // Merge the table with the argument table if provided.
            let Some(target_table) = target_table else {
                return;
            };
            let table = value.as_table();
            let Some(table) = table else {
                print(
                    lua,
                    Verbosity::Warn,
                    &format!(
                        "Script {} did not return a table, so we cannot put its exports into the table provided when calling LoadScript.",
                        file_path.to_string_lossy()
                    ),
                );
                return;
            };

            for pair in table.pairs::<mlua::Value, mlua::Value>() {
                let Ok((key, value)) = pair else { continue };
                let _ = target_table.raw_set(key, value);
            }
        }
    }
}

pub fn stringify_lua_value(value: &mlua::Value) -> String {
    let mut seen = Vec::new();
    stringify_lua_value_helper(value, &mut seen)
}

const UNSAFE_INTERNALS_KEY: &str = "VectarineUnsafeInternal";

pub fn get_internals(lua: &mlua::Lua) -> mlua::Table {
    let globals = lua.globals();
    globals.raw_get(UNSAFE_INTERNALS_KEY).unwrap()
}

pub fn to_lua<T>(lua: &mlua::Lua, value: T) -> mlua::Result<mlua::Value>
where
    T: mlua::IntoLua,
{
    value.into_lua(lua)
}

pub fn merge_lua_tables(source: &mlua::Table, target: &mlua::Table) {
    for pair in source.pairs::<mlua::Value, mlua::Value>().flatten() {
        let (key, value) = pair;
        let _ = target.raw_set(key, value);
    }
}

/// Helper function to allow printing messages from anywhere in Rust as long as you have access to a lua handle.
pub fn print(lua: &Rc<mlua::Lua>, verbosity: Verbosity, msg: &str) {
    let internals = get_internals(lua);
    let print_fn: mlua::Function = internals.raw_get("print").unwrap();
    let _ = print_fn.call::<()>((msg.to_string(), verbosity));
}

fn stringify_lua_value_helper(value: &mlua::Value, seen: &mut Vec<mlua::Value>) -> String {
    if seen.contains(value) {
        return "[circular]".to_string();
    }
    seen.push(value.clone());

    match value {
        mlua::Value::Nil => "nil".to_string(),
        mlua::Value::Boolean(b) => b.to_string(),
        mlua::Value::Integer(i) => i.to_string(),
        mlua::Value::Number(n) => n.to_string(),
        mlua::Value::String(s) => s.to_string_lossy(),
        mlua::Value::Table(table) => format!(
            "{{{}}}",
            table
                .pairs::<mlua::Value, mlua::Value>()
                .map(|pair| {
                    if let Ok((key, value)) = pair {
                        let key_str = stringify_lua_value_helper(&key, seen);
                        let value_str = stringify_lua_value_helper(&value, seen);
                        format!("[{key_str}] = {value_str}")
                    } else {
                        "[error]".to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        mlua::Value::Function(func) => {
            let fninfo = func.info();
            format!(
                "[function: {}:{}]",
                fninfo.name.unwrap_or("anonymous".to_string()),
                fninfo.line_defined.unwrap_or(0)
            )
        }
        mlua::Value::Thread(thread) => {
            let ptr = thread.to_pointer();
            format!("[thread: {ptr:?}]")
        }
        mlua::Value::UserData(userdata) => userdata.to_string().unwrap_or_else(|_| {
            let ptr = userdata.to_pointer();
            format!("[userdata: {ptr:?}]")
        }),
        mlua::Value::LightUserData(lightuserdata) => {
            let ptr = lightuserdata.0;
            format!("[lightuserdata: {ptr:?}]")
        }
        _ => "[unknown]".to_string(),
    }
}
