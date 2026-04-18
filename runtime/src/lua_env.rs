use regex::Regex;
use std::path::PathBuf;
use std::{cell::RefCell, path::Path, rc::Rc};

use vectarine_plugin_sdk::mlua::ObjectLike;

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
pub mod lua_ui;
pub mod lua_vec2;
pub mod lua_vec4;

use crate::console::{print_lua_error, print_warn};
use crate::game_resource::ResourceManager;
use crate::graphics::batchdraw::BatchDraw2d;
use crate::io::IoEnvState;

use crate::metrics::MetricsHolder;

pub const BUILT_IN_MODULES: &[&str] = &[
    "vec", "vec4", "event", "fastlist", "camera", "audio", "tile", "loader", "image", "text",
    "graphics", "screen", "io", "debug", "persist", "resource", "physics", "color", "coord",
    "canvas", "ui",
];

pub const DEPRECATED_MODULES: &[(&str, &str)] = &[(
    "screen",
    "The screen module is deprecated as is being replaced by the ui module. You can use Ui.tabs to have the same behavior. Read the guide about using Uis to organize rendering for more information.",
)];

pub struct LuaHandle {
    pub lua: vectarine_plugin_sdk::mlua::Lua,
    pub project_path: PathBuf,
}

pub struct LuaEnvironment {
    pub lua_handle: Rc<LuaHandle>,
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
        metrics: Rc<RefCell<MetricsHolder>>,
        resources: Rc<ResourceManager>,
    ) -> Self {
        let batch = Rc::new(RefCell::new(batch));
        let lua_options = vectarine_plugin_sdk::mlua::LuaOptions::default();
        // We add everything except:
        // vector as we have our own vector type
        // We'd prefer not to add buffer as we have fastlist which does the same, but for compatibility
        // with existing Luau code, we keep it.
        let lua_libs = vectarine_plugin_sdk::mlua::StdLib::MATH
            | vectarine_plugin_sdk::mlua::StdLib::TABLE
            | vectarine_plugin_sdk::mlua::StdLib::STRING
            | vectarine_plugin_sdk::mlua::StdLib::COROUTINE
            | vectarine_plugin_sdk::mlua::StdLib::UTF8
            | vectarine_plugin_sdk::mlua::StdLib::BUFFER
            | vectarine_plugin_sdk::mlua::StdLib::BIT
            | vectarine_plugin_sdk::mlua::StdLib::DEBUG;
        let gl = batch.borrow().drawing_target.gl().clone();

        let lua = vectarine_plugin_sdk::mlua::Lua::new_with(lua_libs, lua_options)
            .expect("Failed to create Lua");
        lua.set_compiler(
            vectarine_plugin_sdk::mlua::Compiler::new()
                .set_optimization_level(2)
                .set_type_info_level(1),
        );
        let _ = lua.sandbox(false);
        let lua_handle = Rc::new(LuaHandle {
            lua,
            project_path: resources.get_resource_path(),
        });

        // We create a table used to store rust state that is tied to the lua environment, for internal use.
        // An example of such state is the current screen object (from the screen.luau module)
        // This screen can only be set/get from lua using function and direct variable access is not allowed, but it needs to be saved somewhere.
        // Hence an internal table.
        lua_handle
            .lua
            .globals()
            .raw_set(UNSAFE_INTERNALS_KEY, lua_handle.lua.create_table().unwrap())
            .unwrap();

        let env_state = Rc::new(RefCell::new(IoEnvState::default()));

        let persist_module = lua_persist::setup_persist_api(&lua_handle.lua).unwrap();
        register_vectarine_module(&lua_handle.lua, "persist", persist_module);

        let vec2_module = lua_vec2::setup_vec_api(&lua_handle.lua).unwrap();
        register_vectarine_module(&lua_handle.lua, "vec", vec2_module);

        let vec4_module = lua_vec4::setup_vec_api(&lua_handle.lua).unwrap();
        register_vectarine_module(&lua_handle.lua, "vec4", vec4_module);

        let resource_module = lua_handle.lua.create_table().unwrap(); // type-only module
        register_vectarine_module(&lua_handle.lua, "resource", resource_module);

        let fastlist_module =
            lua_fastlist::setup_fastlist_api(&lua_handle.lua, &batch, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "fastlist", fastlist_module);

        let color_module = lua_handle.lua.create_table().unwrap();
        register_vectarine_module(&lua_handle.lua, "color", color_module);

        let coords_module = lua_coord::setup_coords_api(&lua_handle.lua, &gl).unwrap();
        register_vectarine_module(&lua_handle.lua, "coord", coords_module);

        let (event_module, default_events, _event_manager) =
            lua_event::setup_event_api(&lua_handle.lua).unwrap();
        register_vectarine_module(&lua_handle.lua, "event", event_module);

        let canvas_module =
            lua_canvas::setup_canvas_api(&lua_handle.lua, &batch, &env_state, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "canvas", canvas_module);

        let image_module =
            lua_image::setup_image_api(&lua_handle.lua, &batch, &env_state, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "image", image_module);

        let text_module =
            lua_text::setup_text_api(&lua_handle.lua, &batch, &env_state, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "text", text_module);

        let graphics_module =
            lua_graphics::setup_graphics_api(&lua_handle.lua, &batch, &env_state, &resources)
                .unwrap();
        register_vectarine_module(&lua_handle.lua, "graphics", graphics_module);

        let screen_module =
            lua_screen::setup_screen_api(&lua_handle.lua, &batch, &env_state, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "screen", screen_module);

        let io_module = lua_io::setup_io_api(&lua_handle.lua, &env_state).unwrap();
        register_vectarine_module(&lua_handle.lua, "io", io_module);

        let camera_module = lua_camera::setup_camera_api(&lua_handle.lua, &env_state).unwrap();
        register_vectarine_module(&lua_handle.lua, "camera", camera_module);

        let debug_module = lua_debug::setup_debug_api(&lua_handle.lua, &metrics).unwrap();
        register_vectarine_module(&lua_handle.lua, "debug", debug_module);

        let audio_module =
            lua_audio::setup_audio_api(&lua_handle.lua, &env_state, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "audio", audio_module);

        let physics_module = lua_physics::setup_physics_api(&lua_handle.lua).unwrap();
        register_vectarine_module(&lua_handle.lua, "physics", physics_module);

        let tile_module = lua_tile::setup_tile_api(&lua_handle.lua, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "tile", tile_module);

        let loader_module = lua_loader::setup_loader_api(&lua_handle.lua, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "loader", loader_module);

        let ui_module =
            lua_ui::setup_ui_api(&lua_handle.lua, &batch, &env_state, &resources).unwrap();
        register_vectarine_module(&lua_handle.lua, "ui", ui_module);

        let original_require = lua_handle
            .lua
            .globals()
            .get::<vectarine_plugin_sdk::mlua::Function>("require")
            .unwrap();
        add_global_fn(
            &lua_handle.lua,
            "require",
            move |lua, module_name: String| {
                // We provide a custom require with the following features:
                // - Can require @vectarine/* modules (like @vectarine/vec)
                // - Can require files in the script folder by their names.
                if module_name.starts_with("@vectarine/") {
                    for (deprecated_module, message) in DEPRECATED_MODULES {
                        if module_name == format!("@vectarine/{}", deprecated_module) {
                            print_warn(message.to_string());
                        }
                    }

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
            },
        );

        // Add this to Debug module?
        add_global_fn(
            &lua_handle.lua,
            "toString",
            move |_, (arg,): (vectarine_plugin_sdk::mlua::Value,)| {
                let string = stringify_lua_value(&arg);
                Ok(string)
            },
        );

        LuaEnvironment {
            lua_handle,
            env_state,
            batch,
            default_events,
            resources,
            metrics,
        }
    }

    pub fn run_file_and_display_error(&self, file_content: &[u8], file_path: &Path) {
        run_file_and_display_error_from_lua_handle(&self.lua_handle, file_content, file_path, None);
    }
}

#[allow(clippy::unwrap_used)]
pub fn add_global_fn<F, A, R>(lua: &vectarine_plugin_sdk::mlua::Lua, name: &str, func: F)
where
    F: Fn(&vectarine_plugin_sdk::mlua::Lua, A) -> vectarine_plugin_sdk::mlua::Result<R> + 'static,
    A: vectarine_plugin_sdk::mlua::FromLuaMulti,
    R: vectarine_plugin_sdk::mlua::IntoLuaMulti,
{
    lua.globals()
        .set(name, lua.create_function(func).unwrap())
        .unwrap()
}

#[allow(clippy::unwrap_used)]
pub fn add_fn_to_table<F, A, R>(
    lua: &vectarine_plugin_sdk::mlua::Lua,
    table: &vectarine_plugin_sdk::mlua::Table,
    name: &str,
    func: F,
) where
    F: Fn(&vectarine_plugin_sdk::mlua::Lua, A) -> vectarine_plugin_sdk::mlua::Result<R> + 'static,
    A: vectarine_plugin_sdk::mlua::FromLuaMulti,
    R: vectarine_plugin_sdk::mlua::IntoLuaMulti,
{
    table.set(name, lua.create_function(func).unwrap()).unwrap();
}

/// Run the given Lua file content assuming it is at the given path.
/// If the file returns a table, and a target_table is provided, the table will be merged into the target_table.
pub fn run_file_and_display_error_from_lua_handle(
    lua_handle: &LuaHandle,
    file_content: &[u8],
    file_path: &Path,
    target_table: Option<&vectarine_plugin_sdk::mlua::Table>,
) {
    // lua.set_compiler(compiler);
    let lua_chunk = lua_handle.lua.load(file_content);
    // Note: We could change the optimization level of the chunk here (for example, inside the runtime)
    let result = lua_chunk
        .set_name(format!("@{}", file_path.to_string_lossy()))
        .eval::<vectarine_plugin_sdk::mlua::Value>();

    match result {
        Err(error) => {
            print_lua_error_from_error(lua_handle, &error);
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

            for pair in table
                .pairs::<vectarine_plugin_sdk::mlua::Value, vectarine_plugin_sdk::mlua::Value>()
            {
                let Ok((key, value)) = pair else { continue };
                let _ = target_table.raw_set(key, value);
            }
        }
    }
}

pub fn register_vectarine_module(
    lua: &vectarine_plugin_sdk::mlua::Lua,
    name: &'static str,
    module: vectarine_plugin_sdk::mlua::Table,
) {
    if !BUILT_IN_MODULES.contains(&name) {
        panic!(
            "You need to add {} to the BUILT_IN_MODULES list in runtime/src/lua_env.rs to be allowed to register it.",
            name
        );
    }
    lua.register_module(&format!("@vectarine/{}", name), module)
        .expect("Failed to register vectarine module");
}

pub fn stringify_lua_value(value: &vectarine_plugin_sdk::mlua::Value) -> String {
    let mut seen = Vec::new();
    stringify_lua_value_helper(value, &mut seen)
}

pub fn to_lua<T>(
    lua: &vectarine_plugin_sdk::mlua::Lua,
    value: T,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Value>
where
    T: vectarine_plugin_sdk::mlua::IntoLua,
{
    value.into_lua(lua)
}

pub fn merge_lua_tables(
    source: &vectarine_plugin_sdk::mlua::Table,
    target: &vectarine_plugin_sdk::mlua::Table,
) {
    for pair in source
        .pairs::<vectarine_plugin_sdk::mlua::Value, vectarine_plugin_sdk::mlua::Value>()
        .flatten()
    {
        let (key, value) = pair;
        let _ = target.raw_set(key, value);
    }
}

pub fn get_line_and_file_of_error(error: &vectarine_plugin_sdk::mlua::Error) -> (usize, String) {
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

fn stringify_lua_value_helper(
    value: &vectarine_plugin_sdk::mlua::Value,
    seen: &mut Vec<vectarine_plugin_sdk::mlua::Value>,
) -> String {
    if seen.contains(value) && matches!(value, vectarine_plugin_sdk::mlua::Value::Table(_)) {
        return "[circular]".to_string();
    }
    seen.push(value.clone());

    match value {
        vectarine_plugin_sdk::mlua::Value::Nil => "nil".to_string(),
        vectarine_plugin_sdk::mlua::Value::Boolean(b) => b.to_string(),
        vectarine_plugin_sdk::mlua::Value::Integer(i) => i.to_string(),
        vectarine_plugin_sdk::mlua::Value::Number(n) => n.to_string(),
        vectarine_plugin_sdk::mlua::Value::String(s) => s.to_string_lossy(),
        vectarine_plugin_sdk::mlua::Value::Table(table) => format!(
            "{{{}}}",
            table
                .pairs::<vectarine_plugin_sdk::mlua::Value, vectarine_plugin_sdk::mlua::Value>()
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
        vectarine_plugin_sdk::mlua::Value::Function(func) => {
            let fninfo = func.info();
            format!(
                "[function: {}:{}]",
                fninfo.name.unwrap_or("anonymous".to_string()),
                fninfo.line_defined.unwrap_or(0)
            )
        }
        vectarine_plugin_sdk::mlua::Value::Thread(thread) => {
            let ptr = thread.to_pointer();
            format!("[thread: {ptr:?}]")
        }
        vectarine_plugin_sdk::mlua::Value::UserData(userdata) => {
            userdata.to_string().unwrap_or_else(|_| {
                let ptr = userdata.to_pointer();
                format!("[userdata: {ptr:?}]")
            })
        }
        vectarine_plugin_sdk::mlua::Value::LightUserData(lightuserdata) => {
            let ptr = lightuserdata.0;
            format!("[lightuserdata: {ptr:?}]")
        }
        _ => "[unknown]".to_string(),
    }
}

const UNSAFE_INTERNALS_KEY: &str = "Vectarine_Unsafe_Internal";

pub fn get_internals(lua: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Table {
    let globals = lua.globals();
    globals
        .raw_get(UNSAFE_INTERNALS_KEY)
        .expect("Failed to get lua internal table")
}

pub fn is_valid_data_type<T: 'static>(value: &vectarine_plugin_sdk::mlua::Value) -> bool {
    match value {
        vectarine_plugin_sdk::mlua::Value::UserData(ud) => ud.is::<T>(),
        _ => false,
    }
}

pub fn print_lua_error_from_error(
    lua_handle: &LuaHandle,
    error: &vectarine_plugin_sdk::mlua::Error,
) {
    #[cfg(feature = "editor")]
    pub fn extract_file_lines_from_error(
        lua_handle: &LuaHandle,
        file_path: &str,
        line: usize,
    ) -> [String; 5] {
        let content = std::fs::read_to_string(lua_handle.project_path.join(file_path));
        let Ok(content) = content else {
            return Default::default();
        };
        let mut content = content
            .lines()
            .skip(line - 3) // lines are 0-indexed, but error.line is 1-indexed
            .take(5);

        [(); 5].map(|_| content.next().unwrap_or_default().to_string())
    }
    #[cfg(not(feature = "editor"))]
    pub fn extract_file_lines_from_error(
        _lua_handle: &LuaHandle,
        _file_path: &str,
        _line: usize,
    ) -> [String; 5] {
        println!("default");
        Default::default()
    }

    let error_msg = error.to_string();
    let (line, file_path) = get_line_and_file_of_error(error);
    let line_content = extract_file_lines_from_error(lua_handle, &file_path, line);
    print_lua_error(error_msg, file_path, line, line_content);
}
