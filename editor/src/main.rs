#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, sync::mpsc::channel};

use egui_sdl2_platform::sdl2::event::{Event, WindowEvent};
use glow::HasContext;
use runtime::{
    RenderingBlock, egui_glow,
    game::drawable_screen_size,
    init_sdl,
    io::{localfs::LocalFileSystem, time::now_ms},
    sound::init_sound_system,
};

use crate::{
    editorconfig::WindowStyle,
    editorinterface::{EditorState, clear_and_draw_when_no_game},
    reload::reload_assets_if_needed,
};

pub mod buildinfo;
pub mod editorconfig;
pub mod editorextrawindow;
pub mod editorinterface;
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
        gl_context,
    } = init_sdl(|video_subsystem| unsafe {
        egui_glow::painter::Context::from_loader_function(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        })
    });
    init_sound_system(&sdl);

    let (editor_window, mut editor_interface) =
        editorextrawindow::create_specific_editor_window(&video.borrow(), &gl);

    let (debounce_event_sender, debounce_receiver) = channel();

    let mut painter =
        egui_glow::Painter::new(gl.clone(), "", None, true).expect("Failed to create painter");

    let mut platform = egui_sdl2_platform::Platform::new(drawable_screen_size(&window.borrow()))
        .expect("Failed to create platform");

    let mut editor_state = EditorState::new(
        video.clone(),
        window.clone(),
        gl.clone(),
        editor_window,
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
        let (game_window_events, editor_window_events): (Vec<_>, Vec<_>) = latest_events
            .into_iter()
            .partition(|e| e.get_window_id() == Some(editor_state.window.borrow().id()));

        window
            .borrow_mut()
            .gl_make_current(&gl_context)
            .expect("Failed to make context current");

        if window.borrow().is_minimized() {
            // Preserve CPU when minimized
            clear_and_draw_when_no_game(&gl);
            window.borrow().gl_swap_window();
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        let now_instant = now_ms();
        let delta_duration =
            std::time::Duration::from_micros(((now_instant - start_of_frame) * 1000.0) as u64);
        start_of_frame = now_instant;

        // Handle basic events
        editorinterface::handle_close_events(&game_window_events);
        editorinterface::handle_close_events(&editor_window_events);

        if let Some(project) = editor_state.project.borrow_mut().as_mut() {
            let game = &mut project.game;

            game.load_resource_as_needed();
            reload_assets_if_needed(
                &gl,
                &game.lua_env.resources,
                &game.lua_env,
                &debounce_receiver,
            );

            window
                .borrow_mut()
                .gl_make_current(&gl_context)
                .expect("Failed to make context current");
            unsafe {
                let window_size = window.borrow().size();
                gl.viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
            }

            // Render the game
            game.main_loop(&game_window_events, &window, delta_duration, true);
        } else {
            // Clear the screen when no project is loaded
            {
                window
                    .borrow()
                    .gl_make_current(&gl_context)
                    .expect("Failed to make context current");
                clear_and_draw_when_no_game(&gl);
            }
            {
                editor_state
                    .editor_specific_window
                    .gl_make_current(&gl_context)
                    .expect("Failed to make context current");
                clear_and_draw_when_no_game(&gl);
            }
        }

        let window_style = editor_state.config.borrow().window_style;

        match window_style {
            WindowStyle::GameSeparateFromEditor => {
                // We finished drawing the game. If it is separate from the editor, we can swap.
                window.borrow().gl_swap_window();
                editorextrawindow::render_editor_in_extra_window(
                    &sdl,
                    &gl,
                    &gl_context,
                    &mut editor_state,
                    &mut editor_interface,
                    &editor_window_events,
                );
                editor_state.editor_specific_window.gl_swap_window();
            }
            WindowStyle::GameWithEditor => {
                editor_state.editor_specific_window.hide();
                window
                    .borrow()
                    .gl_make_current(&gl_context)
                    .expect("Failed to make context current");
                editor_state.draw_editor_interface(
                    &mut platform,
                    &sdl,
                    &game_window_events,
                    &mut painter,
                );
                window.borrow().gl_swap_window();
            }
        }
    }
}
