use std::cell::RefCell;
use std::fmt::Display;
use std::fs;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use runtime::anyhow;
use runtime::console;
use runtime::console::ConsoleMessage;
use runtime::game::Game;
use runtime::glow::HasContext;
use runtime::glow::PixelPackData;
use runtime::inithelpers::RenderingBlock;
use runtime::inithelpers::set_opengl_attributes;
use runtime::io::localfs::LocalFileSystem;
use runtime::projectinfo::get_project_info;
use vectarine_plugin_sdk::glow;
use vectarine_plugin_sdk::sdl2;
use vectarine_plugin_sdk::sdl2::video::{SwapInterval, Window};

pub fn init_sdl_headless<F>(make_gl_from_video_system: F) -> RenderingBlock
where
    F: FnOnce(&sdl2::VideoSubsystem) -> glow::Context,
{
    let sdl_context = sdl2::init().expect("Failed to initialize SDL");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to initialize video subsystem");
    let gl_attr = video_subsystem.gl_attr();
    set_opengl_attributes(gl_attr);

    let window: Window = video_subsystem
        .window("Vectarine", 800, 600)
        .opengl()
        .hidden()
        .allow_highdpi() // For Retina displays on macOS
        .build()
        .expect("Failed to create window");

    let event_pump = sdl_context
        .event_pump()
        .expect("Failed to create event pump");

    let gl_context = ManuallyDrop::new(
        window
            .gl_create_context()
            .expect("Failed to create GL context"),
    );

    let gl = make_gl_from_video_system(&video_subsystem);
    let gl: Arc<glow::Context> = Arc::new(gl);

    let _ = video_subsystem.gl_set_swap_interval(SwapInterval::VSync);

    RenderingBlock {
        sdl: sdl_context,
        video: Rc::new(video_subsystem),
        window: Rc::new(RefCell::new(window)),
        event_pump,
        gl_context,
        gl,
    }
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
    window: Rc<RefCell<Window>>,
}

impl GameHeadlessRunner {
    pub fn new(project_path: &Path) -> vectarine_plugin_sdk::anyhow::Result<Self> {
        let RenderingBlock {
            sdl: _sdl,
            video,
            window,
            event_pump: _event_pump,
            gl,
            ..
        } = init_sdl_headless(|video_subsystem| unsafe {
            glow::Context::from_loader_function(|name| {
                video_subsystem.gl_get_proc_address(name) as *const _
            })
        });

        // init_sound_system(&sdl); // headless mode does not simulate sound.
        // init_fs(); // headless mode does not run in a browser, so no need to init IDBFS.

        let local_fs = Box::new(LocalFileSystem);

        println!("Loading project from {:?}", project_path);

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

        let result = Game::from_project_safe_sync(
            project_path,
            &project_info,
            local_fs,
            gl,
            &video,
            &window,
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

        Ok(GameHeadlessRunner { game, window })
    }

    /// Steps the game forward by the given duration. You can pass a fake duration to see how the game behaves on slow hardware.
    /// You need to pass the events that occurred since last step to simulate user input (you can pass an empty vector.)
    pub fn step(
        &mut self,
        delta_duration: std::time::Duration,
        latest_events: Vec<sdl2::event::Event>,
    ) -> FrameResult {
        self.game.load_resource_as_needed();

        self.game
            .main_loop(latest_events.iter(), &self.window, delta_duration, false);

        let mut logs: Vec<ConsoleMessage> = Vec::new();
        let mut frame_logs: Vec<String> = Vec::new();
        console::consume_logs(|log| {
            logs.push(log);
        });
        console::consume_frame_logs(|log| {
            frame_logs.push(log);
        });
        console::clear_all_logs();

        self.window.borrow().gl_swap_window();

        FrameResult { logs, frame_logs }
    }

    /// Takes a screenshot of the current game state and return the raw RGBA pixel data along with the width and height of the image.
    pub fn screenshot(&self) -> vectarine_plugin_sdk::anyhow::Result<(Vec<u8>, u32, u32)> {
        let (width, height) = self.window.borrow().drawable_size();
        let mut pixel_buffer = vec![0u8; (width * height * 4) as usize];
        unsafe {
            let format = glow::RGBA;
            let gltype = glow::UNSIGNED_BYTE;
            let pixels = PixelPackData::Slice(Some(pixel_buffer.as_mut_slice()));
            self.game
                .gl
                .read_pixels(0, 0, width as i32, height as i32, format, gltype, pixels);
        }
        Ok((pixel_buffer, width, height))
    }
}
