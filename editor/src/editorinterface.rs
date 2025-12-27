use std::{
    cell::RefCell,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

use egui_sdl2_platform::sdl2;
use glow::HasContext;
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache, new_debouncer, notify,
};
use runtime::{
    anyhow::{self},
    game::drawable_screen_size,
    io::{
        fs::{FileSystem, ReadOnlyFileSystem},
        localfs::LocalFileSystem,
    },
    sdl2::video::Window,
    toml,
};

use crate::{
    editorconfig::{EDITOR_CONFIG_FILE, EditorConfig},
    editorinterface::emptyscreen::draw_empty_screen,
    egui_sdl2_platform,
    exportinterface::draw_editor_export,
    projectstate::ProjectState,
};
use editorconsole::draw_editor_console;
use editormenu::draw_editor_menu;
use editorprofiler::draw_editor_profiler;
use editorresources::draw_editor_resources;
use editorwatcher::draw_editor_watcher;

pub mod editorconsole;
pub mod editormenu;
pub mod editorprofiler;
pub mod editorresources;
pub mod editorwatcher;
pub mod emptyscreen;

pub struct EditorState {
    pub config: Rc<RefCell<EditorConfig>>,
    pub text_command: String,

    pub project: Rc<RefCell<Option<ProjectState>>>,

    pub start_time: std::time::Instant,
    pub video: Rc<RefCell<sdl2::VideoSubsystem>>,
    pub window: Rc<RefCell<sdl2::video::Window>>,
    pub gl: Arc<glow::Context>,

    pub editor_window: sdl2::video::Window,
    pub editor_interface: EditorInterfaceWithGl,
    debouncer: Rc<RefCell<Debouncer<notify::RecommendedWatcher, RecommendedCache>>>,
}

impl EditorState {
    pub fn save_config(&self) {
        let config = &self.config.borrow();
        let data = toml::to_string(config.deref()).unwrap_or_default();
        LocalFileSystem.write_file(EDITOR_CONFIG_FILE, data.as_bytes(), Box::new(|_| {}));
    }

    /// Load the editor config from file.
    /// If `auto_start_project` is true, and there was a project opened previously, it is loaded automatically overwriting any current project.
    pub fn load_config(&self, auto_start_project: bool) {
        let config_store = self.config.clone();
        let project = self.project.clone();
        let gl = self.gl.clone();
        let video = self.video.clone();
        let window = self.window.clone();
        let debouncer = self.debouncer.clone();

        LocalFileSystem.read_file(
            EDITOR_CONFIG_FILE,
            Box::new(move |data: Option<Vec<u8>>| {
                let Some(data) = data else {
                    return; // no config file
                };
                if let Ok(config) = toml::from_slice::<EditorConfig>(data.as_slice()) {
                    let previous_project_path = config.opened_project_path.clone();
                    if let Some(project_path_str) = &previous_project_path {
                        let previous_project_path = PathBuf::from(project_path_str);
                        let parent = previous_project_path.parent();
                        if let Some(parent) = parent {
                            let _ = debouncer.borrow_mut().unwatch(parent);
                        }
                    }

                    *config_store.borrow_mut() = config;
                    if auto_start_project
                        && let Some(project_path_str) = &config_store.borrow().opened_project_path
                    {
                        let project_path = PathBuf::from(project_path_str);
                        let parent = project_path.parent();
                        if let Some(parent) = parent {
                            let _ = debouncer
                                .borrow_mut()
                                .watch(parent, notify::RecursiveMode::Recursive);
                        }

                        ProjectState::new(
                            &project_path,
                            Box::new(LocalFileSystem),
                            gl,
                            video,
                            window,
                            |loaded_project| {
                                if let Ok(loaded_project) = loaded_project {
                                    project.replace(Some(loaded_project));
                                }
                            },
                        );
                    }
                }
            }),
        );
    }

    pub fn new(
        video: Rc<RefCell<sdl2::VideoSubsystem>>,
        window: Rc<RefCell<sdl2::video::Window>>,
        gl: Arc<glow::Context>,
        editor_window: sdl2::video::Window,
        editor_interface: EditorInterfaceWithGl,
        debounce_event_sender: mpsc::Sender<DebouncedEvent>,
    ) -> Self {
        Self {
            config: Rc::new(RefCell::new(EditorConfig::default())),
            text_command: String::new(),
            start_time: Instant::now(),
            project: Rc::new(RefCell::new(None)),
            video,
            window,
            gl,
            editor_window,
            editor_interface,
            debouncer: Rc::new(RefCell::new(
                new_debouncer(
                    Duration::from_millis(10),
                    None,
                    move |result: DebounceEventResult| match result {
                        Ok(events) => events.iter().for_each(|event| {
                            let _ = debounce_event_sender.send(event.clone());
                        }),
                        Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
                    },
                )
                .expect("Failed to create debouncer"),
            )),
        }
    }

    pub fn load_project<F>(
        &self,
        file_system: Box<dyn ReadOnlyFileSystem>,
        project_path: &Path,
        callback: F,
    ) where
        F: FnOnce(anyhow::Result<()>),
    {
        ProjectState::new(
            project_path,
            file_system,
            self.gl.clone(),
            self.video.clone(),
            self.window.clone(),
            |project| {
                match project {
                    Ok(p) => self.project.borrow_mut().replace(p),
                    Err(e) => {
                        callback(Err(e));
                        return;
                    }
                };
                self.config.borrow_mut().opened_project_path =
                    Some(project_path.to_string_lossy().to_string());

                let parent = project_path.parent();
                if let Some(parent) = parent {
                    // This only makes sense for a local file system.
                    let _ = self
                        .debouncer
                        .borrow_mut()
                        .watch(parent, notify::RecursiveMode::Recursive);
                }
                self.save_config();
                callback(Ok(()));
            },
        );
    }

    pub fn reload_project(&mut self) {
        if let Some(proj) = &mut *self.project.borrow_mut() {
            proj.reload();
        }
    }

    pub fn close_project(&mut self) {
        if let Some(proj) = &*self.project.borrow() {
            let project_path = &proj.project_path;
            let parent = project_path.parent();
            if let Some(parent) = parent {
                let _ = self.debouncer.borrow_mut().unwatch(parent);
            }
        }

        self.project.borrow_mut().take();
        self.config.borrow_mut().opened_project_path = None;
        self.save_config();
    }

    pub fn draw_editor_interface(
        &mut self,
        platform: &mut egui_sdl2_platform::Platform,
        sdl: &sdl2::Sdl,
        latest_events: &[sdl2::event::Event],
        painter: &mut egui_glow::Painter,
    ) {
        // Update the time
        platform.update_time(self.start_time.elapsed().as_secs_f64());
        // Get the egui context and begin drawing the frame
        let ctx = platform.context();

        draw_editor_menu(self, &ctx);
        draw_editor_console(self, &ctx);
        draw_editor_resources(self, &ctx);
        draw_editor_watcher(self, &ctx);
        draw_editor_profiler(self, &ctx);
        draw_editor_export(self, &ctx);

        if self.project.borrow().is_none() {
            draw_empty_screen(self, &ctx);
        }

        // Stop drawing the egui frame and get the full output
        let full_output = platform.end_frame(&mut self.video.borrow_mut());
        match full_output {
            Ok(full_output) => {
                // Get the paint jobs
                let paint_jobs = platform.tessellate(&full_output);
                let pj = paint_jobs.as_slice();

                // Render the editor interface on top of the game.
                let size = drawable_screen_size(&self.window.borrow());

                let pixel_per_point = size.0 as f32 / self.window.borrow().size().0 as f32;

                painter.paint_and_update_textures(
                    [size.0, size.1],
                    pixel_per_point,
                    pj,
                    &full_output.textures_delta,
                );
            }
            Err(e) => println!("Failed to render debug ui: {e:?}"),
        };
        for event in latest_events {
            // Convert mouse position.
            platform.handle_event(event, sdl, &self.video.borrow());
        }
    }
}

pub fn process_events_when_no_game(latest_events: &[sdl2::event::Event], gl: &glow::Context) {
    unsafe {
        gl.clear_color(0.1, 0.1, 0.1, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
    }

    for event in latest_events {
        if let sdl2::event::Event::Quit { .. } = event {
            std::process::exit(0);
        }
    }
}

pub struct EditorInterfaceWithGl {
    pub platform: egui_sdl2_platform::Platform,
    pub painter: egui_glow::Painter,
    pub gl: Arc<glow::Context>,
}

pub fn make_gl_context(video_subsystem: &runtime::sdl2::VideoSubsystem) -> glow::Context {
    unsafe {
        egui_glow::painter::Context::from_loader_function(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        })
    }
}

/// Create an SDL2 Window to display the editor without the game.
/// This window is hidden by default and is show when the WindowStyle is set to GameSeparateFromEditor.
pub fn create_specific_editor_window(
    video_subsystem: &runtime::sdl2::VideoSubsystem,
    gl: &Arc<glow::Context>,
) -> (Window, EditorInterfaceWithGl) {
    let editor_window: Window = video_subsystem
        .window("Vectarine Editor", 800, 600)
        .opengl()
        .allow_highdpi() // For Retina displays on macOS
        .resizable()
        // .hidden() // hidden by default
        .build()
        .expect("Failed to create window");
    let interface =
        EditorInterfaceWithGl::new(&editor_window, gl).expect("Failed to create editor interface");
    (editor_window, interface)
}

impl EditorInterfaceWithGl {
    pub fn new(window: &Window, gl: &Arc<glow::Context>) -> anyhow::Result<Self> {
        let painter =
            egui_glow::Painter::new(gl.clone(), "", None, true).expect("Failed to create painter");
        let platform = egui_sdl2_platform::Platform::new(drawable_screen_size(window))
            .expect("Failed to create platform");

        Ok(Self {
            platform,
            painter,
            gl: gl.clone(),
        })
    }
}
