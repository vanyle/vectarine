use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use glow::HasContext;
use sdl2::video::WindowPos;

use crate::{
    console::Verbosity,
    game_resource::{Resource, ResourceId, Status, script_resource::ScriptResource},
    graphics::batchdraw::BatchDraw2d,
    io::{fs::ReadOnlyFileSystem, process_events},
    lua_env::{LuaEnvironment, lua_screen},
    projectinfo::ProjectInfo,
};

pub struct Game {
    pub gl: Arc<glow::Context>,
    pub lua_env: LuaEnvironment,
    pub was_main_script_executed: bool,
    pub main_script_path: String,
}

impl Game {
    /// Creates a new game instance from the given project path.
    /// The game will load resources using the provided file system.
    /// The game provided in the callback is fully initialized and ready to use.
    pub fn from_project<F>(
        project_path: &Path,
        project_info: &ProjectInfo,
        file_system: Box<dyn ReadOnlyFileSystem>,
        gl: Arc<glow::Context>,
        video: &Rc<RefCell<sdl2::VideoSubsystem>>,
        window: &Rc<RefCell<sdl2::video::Window>>,
        callback: F,
    ) where
        F: FnOnce(anyhow::Result<Self>),
    {
        let project_dir = project_path.parent();
        let Some(project_dir) = project_dir else {
            callback(Err(anyhow::anyhow!("Invalid project path")));
            return;
        };

        let _ = window.borrow_mut().set_title(&project_info.title);
        let _ = window.borrow_mut().set_size(
            project_info.default_screen_width,
            project_info.default_screen_height,
        );

        let batch = BatchDraw2d::new(&gl).unwrap();
        let lua_env = LuaEnvironment::new(batch, file_system, project_dir);
        let mut game = Game::from_lua(&gl, lua_env, project_info.main_script_path.clone());
        game.load(video, window);
        let path = Path::new(&game.main_script_path);
        game.lua_env.resources.load_resource::<ScriptResource>(
            path,
            gl.clone(),
            game.lua_env.lua.clone(),
            game.lua_env.default_events.resource_loaded_event,
        );
        callback(Ok(game));
    }

    fn from_lua(
        gl: &Arc<glow::Context>,
        lua_env: LuaEnvironment,
        main_script_path: String,
    ) -> Self {
        Game {
            gl: gl.clone(),
            lua_env,
            was_main_script_executed: false,
            main_script_path,
        }
    }

    /// Initializes the game environment with the current video and window information.
    /// This needs to be called before loading Lua scripts.
    fn load(
        &mut self,
        video: &Rc<RefCell<sdl2::VideoSubsystem>>,
        window: &Rc<RefCell<sdl2::video::Window>>,
    ) {
        // Make screen and window size accessible inside Load.
        if let Ok(display_size) = video.borrow().display_bounds(0) {
            self.lua_env.env_state.borrow_mut().screen_width = display_size.width();
            self.lua_env.env_state.borrow_mut().screen_height = display_size.height();

            let size = screen_size(&window.borrow());
            let drawable_size = drawable_screen_size(&window.borrow());
            let (px_ratio_x, px_ratio_y) = (
                drawable_size.0 as f32 / size.0 as f32,
                drawable_size.1 as f32 / size.1 as f32,
            );

            self.lua_env.env_state.borrow_mut().px_ratio_x = px_ratio_x;
            self.lua_env.env_state.borrow_mut().px_ratio_y = px_ratio_y;
        }

        {
            let (width, height) = screen_size(&window.borrow());
            self.lua_env.env_state.borrow_mut().window_width = width;
            self.lua_env.env_state.borrow_mut().window_height = height;
        }
    }

    pub fn print_to_editor_console(&self, message: &str) {
        self.lua_env.print(message, Verbosity::Info);
    }

    pub fn get_resource_or_print_error<T>(&self, id: ResourceId) -> Option<Rc<T>>
    where
        T: Resource,
    {
        let resource_manager = &self.lua_env.resources;
        let resource = resource_manager.get_by_id::<T>(id);
        let res = match resource {
            Ok(res) => res,
            Err(cause) => {
                self.print_to_editor_console(&format!(
                    "Warning: Failed to get resource with id '{id}': {cause}",
                ));
                return None;
            }
        };
        Some(res)
    }

    pub fn main_loop(
        &mut self,
        events: &[sdl2::event::Event],
        window: &Rc<RefCell<sdl2::video::Window>>,
        delta_time: std::time::Duration,
        _in_editor: bool,
    ) {
        let framebuffer_width;
        let framebuffer_height;
        {
            let mut env_state = self.lua_env.env_state.borrow_mut();
            let (width, height) = drawable_screen_size(&window.borrow());
            env_state.window_width = width;
            env_state.window_height = height;
            let aspect_ratio = width as f32 / height as f32;
            // This works in the editor, but not the runtime.
            // On the web, this is different, the aspect ratio needs to be squared??
            //self.batch.set_aspect_ratio(aspect_ratio * aspect_ratio);

            self.lua_env
                .batch
                .borrow_mut()
                .set_aspect_ratio(aspect_ratio);

            framebuffer_width = width;
            framebuffer_height = height;
        }

        {
            // This is incorrect on the web.
            let gl = &self.gl;
            set_viewport(gl, framebuffer_width, framebuffer_height);
        }

        {
            let env_state = self.lua_env.env_state.borrow_mut();
            if env_state.is_window_resizeable {
                window.borrow_mut().set_resizable(true);
            } else {
                window.borrow_mut().set_resizable(false);
            }
        }
        {
            let mut env_state = self.lua_env.env_state.borrow_mut();
            if let Some(target_size) = env_state.window_target_size {
                let (target_width, target_height) = target_size;
                let _ = window.borrow_mut().set_size(target_width, target_height);
                env_state.window_target_size = None;
            }
            if let Some(fullscreen_request) = env_state.fullscreen_state_request {
                let _ = window.borrow_mut().set_fullscreen(fullscreen_request);
                env_state.fullscreen_state_request = None;
            }
            if let Some(title) = env_state.window_title.take() {
                window.borrow_mut().set_title(&title).unwrap_or(());
            }

            if env_state.center_window_request {
                window
                    .borrow_mut()
                    .set_position(WindowPos::Centered, WindowPos::Centered);
                env_state.center_window_request = false;
            }
        }

        process_events(
            self,
            events,
            framebuffer_width as f32,
            framebuffer_height as f32,
        );

        // Update screen transitions
        lua_screen::update_screen_transition(&self.lua_env.lua, delta_time.as_secs_f32());

        if self.was_main_script_executed {
            let update_fn = self.lua_env.lua.globals().get::<mlua::Function>("Update");
            if let Ok(update_fn) = update_fn {
                let err = update_fn.call::<()>((delta_time.as_secs_f32(),));
                if let Err(err) = err {
                    self.lua_env.print(&err.to_string(), Verbosity::Error);
                }
            } else {
                self.lua_env
                    .print("Update() function not found", Verbosity::Warn);
            }
        }

        {
            self.lua_env
                .batch
                .borrow_mut()
                .draw(&self.lua_env.resources, true);
        }
    }

    /// Calls reload on all unloaded resource inside the manager.
    pub fn load_resource_as_needed(&mut self) {
        let mut to_reload = Vec::new();
        {
            let resource_manager = &self.lua_env.resources;
            for (id, resource) in resource_manager.enumerate() {
                if resource.get_path().display().to_string() == self.main_script_path {
                    self.was_main_script_executed = resource.get_status() == Status::Loaded;
                }
                if resource.get_status() != Status::Unloaded {
                    continue;
                }
                to_reload.push(id);
            }
        }
        for resource_id in to_reload {
            self.lua_env.resources.clone().reload(
                resource_id,
                self.gl.clone(),
                self.lua_env.lua.clone(),
                self.lua_env.default_events.resource_loaded_event,
            );
        }
    }
}

#[cfg(not(target_os = "emscripten"))]
pub fn drawable_screen_size(window: &sdl2::video::Window) -> (u32, u32) {
    window.drawable_size()
}

#[cfg(target_os = "emscripten")]
pub fn drawable_screen_size(_window: &sdl2::video::Window) -> (u32, u32) {
    use emscripten_val::Val;
    // On the web, the drawable size and the screen size are the same.
    let size = Val::global("vectarine").call("getDrawableScreenSize", &[]);
    let width = size.get(&Val::from_str("width")).as_i32();
    let height = size.get(&Val::from_str("height")).as_i32();
    (width as u32, height as u32)
}

#[cfg(not(target_os = "emscripten"))]
pub fn screen_size(window: &sdl2::video::Window) -> (u32, u32) {
    window.size()
}

#[cfg(target_os = "emscripten")]
pub fn screen_size(_window: &sdl2::video::Window) -> (u32, u32) {
    use emscripten_val::Val;
    let size = Val::global("vectarine").call("getScreenSize", &[]);
    let width = size.get(&Val::from_str("width")).as_i32();
    let height = size.get(&Val::from_str("height")).as_i32();
    (width as u32, height as u32)
}

#[cfg(not(target_os = "emscripten"))]
pub fn set_viewport(gl: &glow::Context, width: u32, height: u32) {
    unsafe {
        gl.viewport(0, 0, width as i32, height as i32);
    }
}

#[cfg(target_os = "emscripten")]
pub fn set_viewport(gl: &glow::Context, width: u32, height: u32) {
    unsafe {
        gl.viewport(0, 0, width as i32, height as i32);
    }
}
