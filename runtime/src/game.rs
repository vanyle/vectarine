use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use vectarine_plugin_sdk::glow;
use vectarine_plugin_sdk::glow::HasContext;
use vectarine_plugin_sdk::plugininterface::PluginInterface;
use vectarine_plugin_sdk::sdl2;

use crate::{
    console::print_warn,
    drawing_surface::DrawingSurface,
    game_resource::{
        Resource, ResourceId, ResourceManager, Status, script_resource::ScriptResource,
    },
    graphics::batchdraw::BatchDraw2d,
    io::{fs::ReadOnlyFileSystem, process_events},
    lua_env::LuaEnvironment,
    metrics::{
        DRAW_CALL_METRIC_NAME, LUA_HEAP_SIZE_METRIC_NAME, LUA_SCRIPT_TIME_METRIC_NAME,
        MetricsHolder, TOTAL_FRAME_TIME_METRIC_NAME,
    },
    native_plugin::PluginEnvironment,
    projectinfo::ProjectInfo,
    sound,
};

pub struct Game {
    pub gl: Arc<glow::Context>,
    pub lua_env: LuaEnvironment,
    pub was_main_script_executed: bool,
    pub main_script_path: String,

    pub metrics_holder: Rc<RefCell<MetricsHolder>>,

    pub plugin_env: PluginEnvironment,
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
        window: &Rc<RefCell<dyn DrawingSurface>>,
        callback: F,
    ) where
        F: FnOnce(vectarine_plugin_sdk::anyhow::Result<Self>),
    {
        let project_dir = project_path.parent();
        let Some(project_dir) = project_dir else {
            callback(Err(vectarine_plugin_sdk::anyhow::anyhow!(
                "Invalid project path"
            )));
            return;
        };

        let _ = window.borrow_mut().set_title(&project_info.title);
        let _ = window.borrow_mut().set_drawable_size_in_px(
            project_info.default_screen_width,
            project_info.default_screen_height,
        );

        // Create all the things we need for a game
        let batch = BatchDraw2d::new(&gl).expect("Failed to create batch 2d");
        let metrics = Rc::new(RefCell::new(MetricsHolder::new()));
        let resources = Rc::new(ResourceManager::new(file_system, project_dir));

        PluginEnvironment::load_plugins(
            &project_info.plugins,
            &resources.clone(),
            move |plugin_environment| {
                let lua_env = LuaEnvironment::new(batch, metrics.clone(), resources);

                // Make the game!
                let mut game = Game::from_lua(
                    &gl,
                    lua_env,
                    project_info.main_script_path.clone(),
                    metrics,
                    plugin_environment,
                );

                game.load(window);
                game.plugin_env.init(PluginInterface {
                    lua: &game.lua_env.lua_handle.lua,
                });

                // Load the starting script
                let path = Path::new(&game.main_script_path);
                game.lua_env.resources.load_resource::<ScriptResource>(
                    path,
                    None, // The main script is always loaded with no cause.
                    gl,
                    game.lua_env.lua_handle.clone(),
                    game.lua_env.default_events.resource_loaded_event.clone(),
                );

                // New game means new sounds, so we discard the previous ones (this is useful only for the editor).
                sound::flush_all_samples();

                callback(Ok(game));
            },
        );
    }

    /// Create a new game instance from the given project path without loading any plugins.
    /// We also assume that you have access to a sync filesystem, which is not the case in the browser.
    pub fn from_project_safe_sync(
        project_path: &Path,
        project_info: &ProjectInfo,
        file_system: Box<dyn ReadOnlyFileSystem>,
        gl: Arc<glow::Context>,
        window: &Rc<RefCell<dyn DrawingSurface>>,
        deterministic: bool,
    ) -> vectarine_plugin_sdk::anyhow::Result<Self> {
        // TODO: from_project_safe_sync contains duplicated code with from_project. A refacto would be cool.
        let project_dir = project_path.parent();
        let Some(project_dir) = project_dir else {
            return Err(vectarine_plugin_sdk::anyhow::anyhow!(
                "Invalid project path"
            ));
        };

        let _ = window.borrow_mut().set_title(&project_info.title);
        let _ = window.borrow_mut().set_drawable_size_in_px(
            project_info.default_screen_width,
            project_info.default_screen_height,
        );

        // Create all the things we need for a game
        let batch = BatchDraw2d::new(&gl).expect("Failed to create batch 2d");
        let metrics = Rc::new(RefCell::new(MetricsHolder::new()));
        let resources = Rc::new(ResourceManager::new(file_system, project_dir));

        let lua_env = LuaEnvironment::new(batch, metrics.clone(), resources);

        let mut game = Game::from_lua(
            &gl,
            lua_env,
            project_info.main_script_path.clone(),
            metrics,
            PluginEnvironment::new_empty_environment(),
        );

        if deterministic {
            let chunk = game.lua_env.lua_handle.lua.load("math.randomseed(1)");
            let _ = chunk.exec();
        }

        game.load(window);
        game.plugin_env.init(PluginInterface {
            lua: &game.lua_env.lua_handle.lua,
        });

        // Load the starting script
        let path = Path::new(&game.main_script_path);
        game.lua_env.resources.load_resource::<ScriptResource>(
            path,
            None,
            gl,
            game.lua_env.lua_handle.clone(),
            game.lua_env.default_events.resource_loaded_event.clone(),
        );

        // New game means new sounds, so we discard the previous ones (this is useful only for the editor).
        sound::flush_all_samples();

        Ok(game)
    }

    fn from_lua(
        gl: &Arc<glow::Context>,
        lua_env: LuaEnvironment,
        main_script_path: String,
        metrics_holder: Rc<RefCell<MetricsHolder>>,
        plugin_env: PluginEnvironment,
    ) -> Self {
        Game {
            gl: gl.clone(),
            lua_env,
            was_main_script_executed: false,
            main_script_path,
            metrics_holder,
            plugin_env,
        }
    }

    /// Initializes the game environment with the current video and window information.
    /// This needs to be called before loading Lua scripts.
    fn load(&mut self, window: &Rc<RefCell<dyn DrawingSurface>>) {
        // Make screen and window size accessible inside Load.

        let screen_size = window.borrow().get_screen_size_in_px();

        self.lua_env.env_state.borrow_mut().screen_width = screen_size.0;
        self.lua_env.env_state.borrow_mut().screen_height = screen_size.1;

        self.lua_env.env_state.borrow_mut().px_ratio_x = 1.0; // investigate this, it's strange to have that.
        self.lua_env.env_state.borrow_mut().px_ratio_y = 1.0;

        let (width, height) = window.borrow().get_drawable_size_in_px();
        self.lua_env.env_state.borrow_mut().window_width = width;
        self.lua_env.env_state.borrow_mut().window_height = height;
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
                print_warn(format!(
                    "Warning: Failed to get resource with id '{id}': {cause}",
                ));
                return None;
            }
        };
        Some(res)
    }

    pub fn main_loop<'a>(
        &mut self,
        events: impl Iterator<Item = &'a sdl2::event::Event>,
        window: &Rc<RefCell<dyn DrawingSurface>>,
        delta_time: std::time::Duration,
        _in_editor: bool,
    ) {
        self.lua_env
            .batch
            .borrow()
            .drawing_target
            .reset_draw_call_counter();

        let framebuffer_width;
        let framebuffer_height;
        {
            let mut env_state = self.lua_env.env_state.borrow_mut();
            let (width, height) = window.borrow().get_drawable_size_in_px();
            // imo this is wrong. Having the drawable size is interesting, but we should not mix it with the actual size.
            env_state.window_width = width;
            env_state.window_height = height;
            env_state.is_window_minimized = window.borrow().is_minimized();
            let aspect_ratio = width as f32 / height as f32;

            // This works in the editor, but not the runtime.
            // On the web, this is different, the aspect ratio needs to be squared??
            //self.batch.set_aspect_ratio(aspect_ratio * aspect_ratio);

            // This is wrong: only the size matters, not the drawable size for aspect ratio
            // Size is the actual size, drawable size is about crisp pixels.
            self.lua_env
                .batch
                .borrow_mut()
                .set_aspect_ratio(aspect_ratio);

            framebuffer_width = width;
            framebuffer_height = height;
        }

        {
            // This is incorrect on the web.
            // note that it is right to set this to the drawable size (this is what the viewport is by definition to have an
            // opengl coordinate system)
            let gl = &self.gl;
            set_viewport(gl, framebuffer_width, framebuffer_height);
        }

        {
            sound::update_sound_system()
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
                window
                    .borrow_mut()
                    .set_drawable_size_in_px(target_width, target_height);
                env_state.window_target_size = None;
            }
            if let Some(fullscreen_request) = env_state.fullscreen_state_request {
                window.borrow_mut().set_is_fullscreen(fullscreen_request);
                env_state.fullscreen_state_request = None;
            }
            if let Some(title) = env_state.window_title.take() {
                window.borrow_mut().set_title(&title);
            }

            if env_state.center_window_request {
                window.borrow_mut().center_window();
                env_state.center_window_request = false;
            }
        }

        process_events(
            self,
            events,
            framebuffer_width as f32,
            framebuffer_height as f32,
        );

        // 2D Settings
        unsafe {
            let gl = self.gl.as_ref();
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.disable(glow::DEPTH_TEST);
            // gl.enable(glow::SAMPLE_ALPHA_TO_COVERAGE); // Not needed for 2D.
            gl.enable(glow::MULTISAMPLE);

            // If we don't clear and the game crashes, it looks weird.
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        let plugin_interface = PluginInterface {
            lua: &self.lua_env.lua_handle.lua,
        };
        self.plugin_env.pre_lua_hook(plugin_interface);

        // We poll every frame
        self.lua_env.lua_handle.poll_pending_futures();

        let start_of_lua_update = std::time::Instant::now();
        if self.was_main_script_executed {
            let update_fn = self
                .lua_env
                .lua_handle
                .lua
                .globals()
                .get::<vectarine_plugin_sdk::mlua::Function>("Update");

            if let Ok(update_fn) = update_fn {
                let future = update_fn.call_async::<()>((delta_time.as_secs_f32(),));
                self.lua_env
                    .lua_handle
                    .async_handler
                    .borrow_mut()
                    .execute_frame(&self.lua_env.lua_handle, future, true);
            } else {
                // If there are not futures pending, the update function will never exist.
                // But if there are futures, we might need to wait for them to create the update function.
                if !self.lua_env.lua_handle.are_futures_pending() {
                    print_warn("Update() function not found".to_string());
                } else {
                    // Manually draw a loading screen here (maybe with progress based on the number of pending futures).
                }
            }
        }
        let lua_update_duration = start_of_lua_update.elapsed();

        {
            self.lua_env
                .batch
                .borrow_mut()
                .draw(&self.lua_env.resources, true);
        }

        let plugin_interface = PluginInterface {
            lua: &self.lua_env.lua_handle.lua,
        };
        self.plugin_env.post_lua_hook(plugin_interface);

        // Default Duration metrics
        self.metrics_holder
            .borrow_mut()
            .record_duration_metric(LUA_SCRIPT_TIME_METRIC_NAME, lua_update_duration);
        self.metrics_holder
            .borrow_mut()
            .record_duration_metric(TOTAL_FRAME_TIME_METRIC_NAME, delta_time);

        // Default Counter metrics
        self.metrics_holder.borrow_mut().record_number_metric(
            LUA_HEAP_SIZE_METRIC_NAME,
            self.lua_env.lua_handle.lua.used_memory(),
        );
        self.metrics_holder.borrow_mut().record_number_metric(
            DRAW_CALL_METRIC_NAME,
            self.lua_env
                .batch
                .borrow()
                .drawing_target
                .get_draw_call_counter(),
        );

        self.metrics_holder.borrow_mut().flush();
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
                self.lua_env.lua_handle.clone(),
                self.lua_env.default_events.resource_loaded_event.clone(),
            );
        }
    }
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
