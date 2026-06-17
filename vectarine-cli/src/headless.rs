use std::cell::RefCell;
use std::fs;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use runtime::anyhow;
use runtime::console;
use runtime::game::Game;
use runtime::inithelpers::RenderingBlock;
use runtime::inithelpers::set_opengl_attributes;
use runtime::io::localfs::LocalFileSystem;
use runtime::io::time::now_ms;
use runtime::projectinfo::get_project_info;
use runtime::set_main_loop_wrapper;
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

/// Represents a running instance of a game that is externally controlled.
pub struct GameHeadlessRunner {
    game: Game,
}

pub fn run_game_headless(project_path: &Path) -> Result<GameHeadlessRunner, anyhow::Error> {
    let RenderingBlock {
        sdl,
        video,
        window,
        mut event_pump,
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

    // TODO: instead of running the game here, provide an object with function to step one frame, etc.

    Game::from_project(
        &project_path,
        &project_info,
        local_fs,
        gl,
        &video,
        &window.clone(),
        |result| {
            let Ok(mut game) = result else {
                panic!("Failed to load the game project at {:?}", project_path);
            };
            let mut now = now_ms();

            set_main_loop_wrapper(move || {
                let latest_events = event_pump.poll_iter().collect::<Vec<_>>();
                game.load_resource_as_needed();
                let now_instant = now_ms();
                let delta_duration =
                    std::time::Duration::from_micros(((now_instant - now) * 1000.0) as u64);
                now = now_instant;
                game.main_loop(latest_events.iter(), &window, delta_duration, false);

                // These are for debug and are never displayed in the runtime.
                // We still need to clear them to avoid memory leaks.
                #[allow(unused_variables)]
                {
                    console::consume_logs(|log| {
                        #[cfg(debug_assertions)]
                        println!("{}", log);
                    });
                    console::consume_frame_logs(|log| {
                        #[cfg(debug_assertions)]
                        println!("{}", log);
                    });
                }
                console::clear_all_logs();

                window.borrow().gl_swap_window();
            });
        },
    );

    Ok(GameHeadlessRunner{
        // ...
    })
}
