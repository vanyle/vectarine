#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, sync::mpsc::channel};

use egui_sdl2_platform::sdl2::event::{Event, WindowEvent};
use runtime::{
    RenderingBlock,
    game::drawable_screen_size,
    init_audio, init_sdl,
    io::{localfs::LocalFileSystem, time::now_ms},
};

use crate::{
    editorinterface::{EditorState, process_events_when_no_game},
    reload::reload_assets_if_needed,
};

pub mod editorconsole;
pub mod editorinterface;
pub mod editormenu;
pub mod editorresources;
pub mod editorwatcher;
pub mod egui_sdl2_platform;
pub mod exportinterface;
pub mod projectstate;
pub mod reload;

fn main() {
    gui_main();
}

fn get_project_to_open_from_args() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        let path = PathBuf::from(args[1].clone());
        if path.exists() && path.is_file() {
            return Some(path);
        }
        None
    } else {
        None
    }
}

fn gui_main() {
    let RenderingBlock {
        sdl,
        video,
        window,
        mut event_pump,
        gl,
    } = init_sdl(|video_subsystem| unsafe {
        egui_glow::painter::Context::from_loader_function(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        })
    });
    let _audio_subsystem = init_audio(&sdl);
    // sdl2::mixer::close_audio(); // no need to clean up, the program will clean on exit.

    let (debounce_event_sender, debounce_receiver) = channel();

    // Setup the editor interface
    let mut painter = egui_glow::Painter::new(gl.clone(), "", None, true).unwrap();

    // Create the egui + sdl2 platform
    let mut platform =
        egui_sdl2_platform::Platform::new(drawable_screen_size(&window.borrow())).unwrap();

    let mut editor_state = EditorState::new(
        video.clone(),
        window.clone(),
        gl.clone(),
        debounce_event_sender,
    );

    let project_to_open = get_project_to_open_from_args();
    if let Some(project_path) = project_to_open {
        editor_state.load_config(false);
        editor_state.load_project(Box::new(LocalFileSystem), &project_path, |_r| {});
    } else {
        editor_state.load_config(true);
    }

    window
        .borrow_mut()
        .set_always_on_top(editor_state.config.borrow().is_always_on_top);

    window.borrow_mut().set_resizable(true);

    // Send a fake resize event to egui to initialize drawable area size
    // This is needed on high-DPI screen where the drawable size is greater than window size
    let (width, height) = window.borrow().size();
    let event: Event = Event::Window {
        timestamp: 0,
        window_id: 0,
        win_event: WindowEvent::Resized(width as i32, height as i32),
    };
    platform.handle_event(&event, &sdl, &video.borrow());

    // The main loop
    let mut start_of_frame = now_ms();
    loop {
        let latest_events = event_pump.poll_iter().collect::<Vec<_>>();

        if window.borrow().is_minimized() {
            // Preserve CPU when minimized
            process_events_when_no_game(&latest_events, &gl);
            window.borrow().gl_swap_window();
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        let now_instant = now_ms();
        let delta_duration =
            std::time::Duration::from_micros(((now_instant - start_of_frame) * 1000.0) as u64);
        start_of_frame = now_instant;

        if let Some(project) = editor_state.project.borrow_mut().as_mut() {
            let game = &mut project.game;

            game.load_resource_as_needed();
            reload_assets_if_needed(
                &gl,
                &game.lua_env.resources,
                &game.lua_env,
                &debounce_receiver,
            );

            // Render the game
            game.main_loop(&latest_events, &window, delta_duration, true);
        } else {
            // Clear the screen when no project is loaded
            process_events_when_no_game(&latest_events, &gl);
        }

        editor_state.draw_editor_interface(&mut platform, &sdl, &latest_events, &mut painter);
        window.borrow().gl_swap_window();
    }
}
