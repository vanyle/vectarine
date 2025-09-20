use std::{cell::RefCell, collections::VecDeque, path::Path, rc::Rc};

use mlua::ObjectLike;

pub mod lua_event;
pub mod lua_graphics;
pub mod lua_io;
pub mod lua_resource;
pub mod lua_vec2;

use crate::game_resource::ResourceManager;
use crate::graphics::draw_instruction;
use crate::io::IoEnvState;

#[derive(Debug, Clone)]
pub struct LuaEnvironment {
    pub lua: Rc<mlua::Lua>,
    pub draw_instructions: Rc<RefCell<VecDeque<draw_instruction::DrawInstruction>>>,
    pub env_state: Rc<RefCell<IoEnvState>>,

    pub frame_messages: Rc<RefCell<Vec<String>>>,
    pub messages: Rc<RefCell<VecDeque<String>>>,

    pub resources: Rc<ResourceManager>,
}

impl LuaEnvironment {
    pub fn new() -> Self {
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

        let draw_instructions = Rc::new(RefCell::new(VecDeque::new()));
        let resources = Rc::new(ResourceManager::default());
        let env_state = Rc::new(RefCell::new(IoEnvState::default()));
        let frame_messages = Rc::new(RefCell::new(Vec::new()));
        let messages = Rc::new(RefCell::new(VecDeque::new()));

        lua.globals()
            .set("Global", lua.create_table().unwrap())
            .unwrap();

        lua.globals()
            .set("VectarineUnsafeInternal", lua.create_table().unwrap())
            .unwrap(); // Table used to store unsafe function that we need to access from Rust inside Rust.

        let vec2_module = lua_vec2::setup_vec_api(&lua).unwrap();
        lua.register_module("@vectarine/vec", vec2_module).unwrap();

        let graphics_module =
            lua_graphics::setup_graphics_api(&lua, &draw_instructions, &env_state, &resources)
                .unwrap();
        lua.register_module("@vectarine/graphics", graphics_module)
            .unwrap();

        let io_module = lua_io::setup_io_api(&lua, &env_state, &messages, &frame_messages).unwrap();
        lua.register_module("@vectarine/io", io_module).unwrap();

        let resources_module = lua_resource::setup_resource_api(&lua, &resources).unwrap();
        lua.register_module("@vectarine/resources", resources_module)
            .unwrap();

        let event_module = lua_event::setup_event_api(&lua).unwrap();
        lua.register_module("@vectarine/event", event_module)
            .unwrap();

        // Add this to utils module?
        add_global_fn(&lua, "toString", move |_, (arg,): (mlua::Value,)| {
            let string = stringify_lua_value(&arg);
            Ok(string)
        });

        LuaEnvironment {
            lua,
            draw_instructions,
            env_state,
            frame_messages,
            messages,
            resources,
        }
    }
}

impl Default for LuaEnvironment {
    fn default() -> Self {
        Self::new()
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

pub fn run_file_and_display_error(lua: &LuaEnvironment, file_content: &[u8], file_path: &Path) {
    run_file_and_display_error_from_lua_handle(&lua.lua, file_content, file_path);
}

pub fn run_file_and_display_error_from_lua_handle(
    lua: &Rc<mlua::Lua>,
    file_content: &[u8],
    file_path: &Path,
) {
    let lua_chunk = lua.load(file_content);
    // Note: We could change the optimization level of the chunk here (for example, inside the runtime)
    let result = lua_chunk
        .set_name("@".to_owned() + file_path.to_str().unwrap())
        .exec();
    if result.is_err() {
        let error = result.err().unwrap();
        let error_msg = error.to_string();
        println!("Error: {error_msg} in {}", file_path.to_string_lossy());
    }
}

pub fn stringify_lua_value(value: &mlua::Value) -> String {
    let seen = Vec::new();
    stringify_lua_value_helper(value, seen)
}

pub fn get_internals(lua: &mlua::Lua) -> mlua::Table {
    let globals = lua.globals();
    globals.get("VectarineUnsafeInternal").unwrap()
}

fn stringify_lua_value_helper(value: &mlua::Value, mut seen: Vec<mlua::Value>) -> String {
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
                        let key_str = stringify_lua_value(&key);
                        let value_str = stringify_lua_value(&value);
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
