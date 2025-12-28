use regex::Regex;
use std::{cell::RefCell, path::Path, rc::Rc};

use mlua::ObjectLike;

pub mod lua_audio;
pub mod lua_camera;
pub mod lua_canvas;
pub mod lua_coord;
pub mod lua_debug;
pub mod lua_event;
pub mod lua_fastlist;
pub mod lua_graphics;
pub mod lua_image;
pub mod lua_io;
pub mod lua_loader;
pub mod lua_persist;
pub mod lua_physics;
pub mod lua_resource;
pub mod lua_screen;
pub mod lua_text;
pub mod lua_tile;
pub mod lua_vec2;
pub mod lua_vec4;

use crate::console::{print_lua_error, print_warn};
use crate::game_resource::ResourceManager;
use crate::graphics::batchdraw::BatchDraw2d;
use crate::io::IoEnvState;
use crate::io::fs::ReadOnlyFileSystem;

use crate::metrics::MetricsHolder;

pub struct LuaEnvironment {
    pub lua: Rc<mlua::Lua>,
    pub env_state: Rc<RefCell<IoEnvState>>,

    pub batch: Rc<RefCell<BatchDraw2d>>,

    pub default_events: lua_event::DefaultEvents,

    pub metrics: Rc<RefCell<MetricsHolder>>,
    pub resources: Rc<ResourceManager>,
}

impl LuaEnvironment {
    #[allow(clippy::unwrap_used)]
    pub fn new(
        batch: BatchDraw2d,
        file_system: Box<dyn ReadOnlyFileSystem>,
        base_path: &Path,
        metrics: Rc<RefCell<MetricsHolder>>,
    ) -> Self {
        let batch = Rc::new(RefCell::new(batch));
        let lua_options = mlua::LuaOptions::default();
        let lua_libs = mlua::StdLib::MATH | mlua::StdLib::TABLE | mlua::StdLib::STRING;
        let gl = batch.borrow().drawing_target.gl().clone();

        let lua =
            Rc::new(mlua::Lua::new_with(lua_libs, lua_options).expect("Failed to create Lua"));
        lua.set_compiler(
            mlua::Compiler::new()
                .set_optimization_level(2)
                .set_type_info_level(1),
        );
        let _ = lua.sandbox(false);

        // We create a table used to store rust state that is tied to the lua environment, for internal use.
        // An example of such state is the current screen object (from the screen.luau module)
        // This screen can only be set/get from lua using function and direct variable access is not allowed, but it needs to be saved somewhere.
        // Hence an internal table.
        lua.globals()
            .raw_set(UNSAFE_INTERNALS_KEY, lua.create_table().unwrap())
            .unwrap();

        let resources = Rc::new(ResourceManager::new(file_system, base_path));
        let env_state = Rc::new(RefCell::new(IoEnvState::default()));

        let persist_module = lua_persist::setup_persist_api(&lua).unwrap();
        lua.register_module("@vectarine/persist", persist_module)
            .unwrap();

        let vec2_module = lua_vec2::setup_vec_api(&lua).unwrap();
        lua.register_module("@vectarine/vec", vec2_module).unwrap();

        let vec4_module = lua_vec4::setup_vec_api(&lua).unwrap();
        lua.register_module("@vectarine/vec4", vec4_module).unwrap();

        let fastlist_module = lua_fastlist::setup_fastlist_api(&lua, &batch, &resources).unwrap();
        lua.register_module("@vectarine/fastlist", fastlist_module)
            .unwrap();

        let color_module = lua.create_table().unwrap();
        lua.register_module("@vectarine/color", color_module)
            .unwrap();

        let coords_module = lua_coord::setup_coords_api(&lua, &gl).unwrap();
        lua.register_module("@vectarine/coord", coords_module)
            .unwrap();

        let (event_module, default_events, _event_manager) =
            lua_event::setup_event_api(&lua).unwrap();
        lua.register_module("@vectarine/event", event_module)
            .unwrap();

        let canvas_module =
            lua_canvas::setup_canvas_api(&lua, &batch, &env_state, &resources).unwrap();
        lua.register_module("@vectarine/canvas", canvas_module)
            .unwrap();

        let image_module =
            lua_image::setup_image_api(&lua, &batch, &env_state, &resources).unwrap();
        lua.register_module("@vectarine/image", image_module)
            .unwrap();

        let text_module = lua_text::setup_text_api(&lua, &batch, &env_state, &resources).unwrap();
        lua.register_module("@vectarine/text", text_module).unwrap();

        let graphics_module =
            lua_graphics::setup_graphics_api(&lua, &batch, &env_state, &resources).unwrap();
        lua.register_module("@vectarine/graphics", graphics_module)
            .unwrap();

        let screen_module =
            lua_screen::setup_screen_api(&lua, &batch, &env_state, &resources).unwrap();
        lua.register_module("@vectarine/screen", screen_module)
            .unwrap();

        let io_module = lua_io::setup_io_api(&lua, &env_state).unwrap();
        lua.register_module("@vectarine/io", io_module).unwrap();

        let camera_module = lua_camera::setup_camera_api(&lua, &env_state).unwrap();
        lua.register_module("@vectarine/camera", camera_module)
            .unwrap();

        let debug_module = lua_debug::setup_debug_api(&lua, &metrics).unwrap();
        lua.register_module("@vectarine/debug", debug_module)
            .unwrap();

        let audio_module = lua_audio::setup_audio_api(&lua, &env_state, &resources).unwrap();
        lua.register_module("@vectarine/audio", audio_module)
            .unwrap();

        let physics_module = lua_physics::setup_physics_api(&lua).unwrap();
        lua.register_module("@vectarine/physics", physics_module)
            .unwrap();

        let tile_module = lua_tile::setup_tile_api(&lua, &resources).unwrap();
        lua.register_module("@vectarine/tile", tile_module).unwrap();

        let loader_module = lua_loader::setup_loader_api(&lua, &resources).unwrap();
        lua.register_module("@vectarine/loader", loader_module)
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
            module.raw_set("@vectarine/filename", module_name)?;
            module.raw_set(
                "info",
                "Thank you cowboy! But your module is in another castle!",
            )?;
            // We return an empty table as this is just for the types.
            // We put a message to indicate that. loadScript is what loads the script, not require.
            Ok(module)
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
            default_events,
            resources,
            metrics,
        }
    }

    pub fn run_file_and_display_error(&self, file_content: &[u8], file_path: &Path) {
        run_file_and_display_error_from_lua_handle(&self.lua, file_content, file_path, None);
    }
}

#[allow(clippy::unwrap_used)]
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

#[allow(clippy::unwrap_used)]
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
    // lua.set_compiler(compiler);
    let lua_chunk = lua.load(file_content);
    // Note: We could change the optimization level of the chunk here (for example, inside the runtime)
    let result = lua_chunk
        .set_name(format!("@{}", file_path.to_string_lossy()))
        .eval::<mlua::Value>();

    match result {
        Err(error) => {
            print_lua_error_from_error(&error);
        }
        Ok(value) => {
            // Merge the table with the argument table if provided.
            let Some(target_table) = target_table else {
                return;
            };
            let table = value.as_table();
            let Some(table) = table else {
                print_warn(format!(
                    "Script {} did not return a table, so we cannot put its exports into the table provided when calling LoadScript.",
                    file_path.to_string_lossy()
                ));
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

pub fn get_line_and_file_of_error(error: &mlua::Error) -> (usize, String) {
    let error = error.to_string();
    // An error looks either like this:
    // Some text
    // [C]: in ?
    // path:line: message

    // or like this: syntax error: path:line: message

    if error.starts_with("syntax error") {
        let re = Regex::new(r"syntax error: (.*):([0-9]+): (.*)").expect("The regex is valid");
        let Some(captures) = re.captures(&error) else {
            return (0, "".to_string());
        };
        let Some(line) = captures.get(2) else {
            return (0, "".to_string());
        };
        let line = line.as_str().parse::<usize>().unwrap_or_default();
        let file = captures.get(1).map(|s| s.as_str()).unwrap_or_default();
        return (line, file.to_string());
    }

    let search = "[C]: in ?";
    if let Some(location) = error.find(search) {
        let rest = &error[location + search.len()..].trim_start();
        let re = Regex::new(r"(.*):([0-9]+): (.*)").expect("The regex is valid");
        let Some(captures) = re.captures(rest) else {
            return (0, "".to_string());
        };
        let Some(line) = captures.get(2) else {
            return (0, "".to_string());
        };
        let line = line.as_str().parse::<usize>().unwrap_or_default();
        let file = captures.get(1).map(|s| s.as_str()).unwrap_or_default();
        return (line, file.to_string());
    }

    (0, "".to_string())
}

fn stringify_lua_value_helper(value: &mlua::Value, seen: &mut Vec<mlua::Value>) -> String {
    if seen.contains(value) && matches!(value, mlua::Value::Table(_)) {
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

const UNSAFE_INTERNALS_KEY: &str = "Vectarine_Unsafe_Internal";

pub fn get_internals(lua: &mlua::Lua) -> mlua::Table {
    let globals = lua.globals();
    globals
        .raw_get(UNSAFE_INTERNALS_KEY)
        .expect("Failed to get lua internal table")
}

pub fn is_valid_data_type<T: 'static>(value: &mlua::Value) -> bool {
    match value {
        mlua::Value::UserData(ud) => ud.is::<T>(),
        _ => false,
    }
}

pub fn print_lua_error_from_error(error: &mlua::Error) {
    let error_msg = error.to_string();
    let (line, file_path) = get_line_and_file_of_error(error);
    print_lua_error(error_msg, file_path, line);
}
