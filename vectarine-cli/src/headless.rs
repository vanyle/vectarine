use std::cell::RefCell;
use std::fmt::Display;
use std::fs;
use std::future::Future;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll, Waker};

use runtime::anyhow;
use runtime::console;
use runtime::console::ConsoleMessage;
use runtime::drawing_surface::DrawingSurface;
use runtime::drawing_surface::SurfaceMargins;
use runtime::drawing_surface::sdl_drawing_surface::SdlDrawingSurface;
use runtime::game::Game;
use runtime::gamepad::DummyGamepadSystem;
use runtime::graphics::gfx::Gfx;
use runtime::graphics::gpu::Gpu;
use runtime::graphics::surface::OffscreenSurface;
use runtime::io::localfs::LocalFileSystem;
use runtime::projectinfo::get_project_info;
use vectarine_plugin_sdk::sdl2;
use vectarine_plugin_sdk::sdl2::video::Window;

/// The window of a game that runs without a screen. It is never shown: the game is drawn on an image instead.
pub fn init_sdl_headless() -> Rc<RefCell<SdlDrawingSurface>> {
    let sdl_context = sdl2::init().expect("Failed to initialize SDL");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to initialize video subsystem");

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .hidden()
        //    .allow_highdpi() // Disallow high DPI for consistency in screenshots and testing regardless of the platform.
        .build()
        .expect("Failed to create window");

    Rc::new(RefCell::new(SdlDrawingSurface::new(
        window,
        &video_subsystem,
        Some((1.0, 1.0)),
    )))
}

/// The graphics card is shared by all the games that this process runs.
fn headless_gpu() -> anyhow::Result<Arc<Gpu>> {
    static GPU: OnceLock<Result<Arc<Gpu>, String>> = OnceLock::new();
    GPU.get_or_init(Gpu::headless)
        .clone()
        .map_err(|e| anyhow::anyhow!(e))
}

/// Represents information about what happened during a single frame of the game (logs, Lua errors, etc.)
pub struct FrameResult {
    pub logs: Vec<ConsoleMessage>,
    pub frame_logs: Vec<String>,
}

impl FrameResult {
    pub fn has_errors(&self) -> bool {
        self.logs
            .iter()
            .any(|log| matches!(log, ConsoleMessage::Error(_)))
    }
}

impl Display for FrameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for log in &self.logs {
            match log {
                ConsoleMessage::Info(msg) => writeln!(f, "[LOG] {}", msg)?,
                ConsoleMessage::Warning(msg) => writeln!(f, "[WARN] {}", msg)?,
                ConsoleMessage::Error(msg) => writeln!(f, "[ERROR] {}", msg)?,
                ConsoleMessage::LuaError(msg) => writeln!(f, "[ERROR] {}", msg)?,
                ConsoleMessage::Reload => writeln!(f, "--- Reload ---")?,
            }
        }
        writeln!(f, "--- Frame logs ---")?;
        for frame_log in &self.frame_logs {
            writeln!(f, "{}", frame_log)?;
        }
        Ok(())
    }
}

/// Represents a running instance of a game that is externally controlled.
///
/// You can use this to step the game forward, inspect it, take screenshots, etc.
pub struct GameHeadlessRunner {
    game: Game,
    drawing_surface: Rc<RefCell<SdlDrawingSurface>>,
    gfx: Gfx,
    /// The image that the game is drawn on, in place of the window
    surface: OffscreenSurface,
}

impl GameHeadlessRunner {
    pub fn new(project_path: &Path) -> vectarine_plugin_sdk::anyhow::Result<Self> {
        let window = init_sdl_headless();
        let gfx = Gfx::new(headless_gpu()?);
        let (width, height) = window.borrow().get_drawable_size_in_hardware_px();
        let surface = OffscreenSurface::new(gfx.gpu(), width, height);

        // init_sound_system(&sdl); // headless mode does not simulate sound.
        // init_fs(); // headless mode does not run in a browser, so no need to init IDBFS.

        let local_fs = Box::new(LocalFileSystem);

        let Ok(project_manifest_content) = fs::read_to_string(project_path) else {
            return Err(anyhow::anyhow!(
                "Failed to read the project manifest at {:?}",
                project_path
            ));
        };

        let Ok(project_info) = get_project_info(&project_manifest_content) else {
            return Err(anyhow::anyhow!(
                "Failed to parse the project manifest at {:?}",
                project_path
            ));
        };

        let drawing_surface =
            window.clone() as Rc<RefCell<dyn runtime::drawing_surface::DrawingSurface>>;
        let result = Game::from_project_safe_sync(
            project_path,
            &project_info,
            local_fs,
            gfx.clone(),
            &drawing_surface,
            Rc::new(DummyGamepadSystem),
            true,
        );

        let game = match result {
            Ok(game) => game,
            Err(err) => {
                return Err(anyhow::anyhow!(
                    "Failed to load the game project at {:?}: {}",
                    project_path,
                    err
                ));
            }
        };

        Ok(GameHeadlessRunner {
            game,
            drawing_surface: window,
            gfx,
            surface,
        })
    }

    pub fn window_id(&self) -> u32 {
        self.drawing_surface.borrow().window.id()
    }

    pub fn run_lua_code(
        &mut self,
        code: &str,
    ) -> vectarine_plugin_sdk::anyhow::Result<vectarine_plugin_sdk::mlua::Value> {
        /// After this many attempts, the code is probably waiting for something that never happens.
        const MAX_POLLS: usize = 1000;

        let function = self
            .game
            .lua_env
            .lua_handle
            .lua
            .load(code)
            .set_name("test_code")
            .into_function()
            .map_err(|e| anyhow::anyhow!("Failed to run Lua code: {}", e))?;
        let mut future = Box::pin(function.call_async::<vectarine_plugin_sdk::mlua::Value>(()));
        let mut context = Context::from_waker(Waker::noop());

        for _ in 0..MAX_POLLS {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(result) => {
                    return result.map_err(|e| anyhow::anyhow!("Failed to run Lua code: {}", e));
                }
                Poll::Pending => {
                    // Same as at the start of a frame: load the requested resources, then advance the futures
                    // of the scripts being executed (a required script can require other scripts).
                    self.game.load_resource_as_needed();
                    self.game.lua_env.lua_handle.poll_pending_futures();
                }
            }
        }
        Err(anyhow::anyhow!(
            "Failed to run Lua code: it was still waiting for an asynchronous function (like require) after {} attempts",
            MAX_POLLS
        ))
    }

    /// Steps the game forward by the given duration. You can pass a fake duration to see how the game behaves on slow hardware.
    /// You need to pass the events that occurred since last step to simulate user input (you can pass an empty vector.)
    pub fn step(
        &mut self,
        delta_duration: std::time::Duration,
        latest_events: &[sdl2::event::Event],
    ) -> FrameResult {
        self.game.load_resource_as_needed();

        // Sadly, it looks like this clone is required.
        let drawing_surface: Rc<RefCell<dyn DrawingSurface>> = self.drawing_surface.clone();
        let (width, height) = drawing_surface.borrow().get_drawable_size_in_hardware_px();
        self.surface.begin_frame(&self.gfx, width, height);
        self.game.main_loop(
            latest_events.iter(),
            &drawing_surface,
            delta_duration,
            false,
            SurfaceMargins::default(),
        );

        self.surface.end_frame(&self.gfx);
        self.take_logs()
    }

    /// Returns what the game logged since the last call (or the last step), and clears it.
    pub fn take_logs(&mut self) -> FrameResult {
        let mut logs: Vec<ConsoleMessage> = Vec::new();
        let mut frame_logs: Vec<String> = Vec::new();
        console::consume_logs(|log| {
            logs.push(log);
        });
        console::consume_frame_logs(|log| {
            frame_logs.push(log);
        });
        console::clear_all_logs();
        FrameResult { logs, frame_logs }
    }

    /// Takes a screenshot of the current game state and return the raw RGBA pixel data along with the width and height of the image.
    /// The rows of the image go from its top to its bottom.
    pub fn screenshot(&self) -> vectarine_plugin_sdk::anyhow::Result<(Vec<u8>, u32, u32)> {
        self.surface
            .read_pixels(self.gfx.gpu())
            .map_err(|e| anyhow::anyhow!(e))
    }
}
