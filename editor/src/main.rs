#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, ops::Deref, path::PathBuf, rc::Rc, sync::mpsc::channel};

use runtime::{
    drawing_surface::{DrawingSurface, SurfaceMargins},
    egui_wgpu, init_sdl,
    inithelpers::RenderingBlock,
    io::{localfs::LocalFileSystem, time::now_ms},
    sound::init_sound_system,
};

use crate::{
    editorconfig::WindowStyle,
    editorextrawindow::{
        draw_error_in_game_window, draw_info_in_empty_game_window, send_window_resize_sync_event,
    },
    editorinterface::{EditorState, clear_window},
    reload::reload_assets_if_needed,
};

pub mod editorconfig;
pub mod editorextrawindow;
pub mod editorinterface;
pub mod egui_sdl2_platform;
pub mod export;
pub mod game_drawing_surface;
pub mod luau;
pub mod pluginsystem;
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
        video_subsystem,
        window,
        mut event_pump,
        gfx,
        surface: mut game_window_surface,
        gamepad_system,
    } = init_sdl();
    init_sound_system(&sdl);

    let (editor_window, mut editor_interface) =
        editorextrawindow::create_specific_editor_window(&video_subsystem, &gfx);

    let (debounce_event_sender, debounce_receiver) = channel();

    let mut painter = egui_wgpu::Renderer::new(
        &gfx.gpu().device,
        game_window_surface.format(),
        egui_wgpu::RendererOptions::default(),
    );

    // The "screen_size" provided needs to be the size of the viewport.
    let mut platform =
        egui_sdl2_platform::Platform::new(window.borrow().get_drawable_size_in_hardware_px())
            .expect("Failed to create platform");

    let mut editor_state = EditorState::new(
        window.clone(),
        gfx.clone(),
        editor_window,
        debounce_event_sender,
        &video_subsystem,
        gamepad_system,
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
        .window
        .set_always_on_top(editor_state.config.borrow().is_always_on_top);

    window.borrow_mut().set_resizable(true);

    // Send a fake resize event to egui to initialize drawable area size
    // This is needed on high-DPI screen where the drawable size is greater than window size
    // Or is it??
    send_window_resize_sync_event(
        &sdl,
        &video_subsystem,
        &window.borrow().window,
        &mut platform,
        editor_state.window.borrow().deref(),
    );
    send_window_resize_sync_event(
        &sdl,
        &video_subsystem,
        &editor_state.editor_specific_window_surface.borrow().window,
        &mut editor_interface.platform,
        editor_state.window.borrow().deref(),
    );

    // The main loop
    let mut start_of_frame = now_ms();
    let mut game_margin_top = 0.0;

    loop {
        let latest_events = event_pump.poll_iter().collect::<Vec<_>>();
        let (game_window_events, editor_window_events): (Vec<_>, Vec<_>) = latest_events
            .into_iter()
            .partition(|e| e.get_window_id() == Some(editor_state.window.borrow().window.id()));

        if window.borrow().is_minimized() {
            // Preserve CPU when minimized
            std::thread::sleep(std::time::Duration::from_millis(200));
            if let Some(project) = editor_state.project.borrow_mut().as_mut() {
                // Ignore infinite loop errors while minimized: do not track frame timings.
                *project.hook_error.borrow_mut() = None;
                *project.hook_timing.borrow_mut() = None;
            }
            continue;
        }

        let now_instant = now_ms();
        let delta_duration =
            std::time::Duration::from_micros(((now_instant - start_of_frame) * 1000.0) as u64);
        start_of_frame = now_instant;

        // Handle basic events
        editorinterface::handle_close_events(&game_window_events);
        editorinterface::handle_close_events(&editor_window_events);

        let window_style = editor_state.config.borrow().window_style;

        let (width, height) = window.borrow().get_drawable_size_in_hardware_px();
        let mut is_something_shown = game_window_surface.begin_frame(&gfx, width, height);
        clear_window(&gfx);

        if let Some(project) = editor_state.project.borrow_mut().as_mut() {
            let game = &mut project.game;
            luau::update_hook_timing(&project.hook_timing); // Setup infinite loop detection early in the frame.

            game.load_resource_as_needed();
            let script_reloaded = reload_assets_if_needed(
                &gfx,
                &game.lua_env.resources,
                &game.lua_env,
                &debounce_receiver,
            );

            if script_reloaded {
                *project.hook_error.borrow_mut() = None;
            }

            let ref_game_drawing_surface =
                editor_state.game_drawing_surface.clone() as Rc<RefCell<dyn DrawingSurface>>;
            let margins = SurfaceMargins {
                left: 0.0,
                right: 0.0,
                top: game_margin_top,
                bottom: 0.0,
            };
            if let Some(ref error) = *project.hook_error.borrow() {
                let game_drawing_surface = ref_game_drawing_surface.borrow();
                gfx.set_viewport(game_drawing_surface.viewport(margins));
                draw_error_in_game_window(
                    &gfx,
                    &*game_drawing_surface,
                    &mut editor_state.editor_batch_draw,
                    error,
                );
            } else {
                let game_events = editorinterface::filter_events(
                    &game_window_events,
                    editor_state.editor_want_mouse,
                    editor_state.editor_want_keyboard,
                );

                game.main_loop(
                    game_events,
                    &ref_game_drawing_surface,
                    delta_duration,
                    true,
                    margins,
                );
            }
        } else if window_style == WindowStyle::GameSeparateFromEditor {
            draw_info_in_empty_game_window(
                &gfx,
                &window.borrow().window,
                &mut editor_state.editor_batch_draw,
            );
        }

        match window_style {
            WindowStyle::GameSeparateFromEditor => {
                // We finished drawing the game. If it is separate from the editor, we can show it.
                game_window_surface.end_frame(&gfx);

                game_margin_top = 0.0;
                is_something_shown |= editorextrawindow::render_editor_in_extra_window(
                    &sdl,
                    &video_subsystem,
                    &gfx,
                    &mut editor_state,
                    &mut editor_interface,
                    &editor_window_events,
                );
            }
            WindowStyle::GameWithEditor => {
                editor_state
                    .editor_specific_window_surface
                    .borrow_mut()
                    .window
                    .hide();

                gfx.apply_pending_clear();
                let target = game_window_surface.frame_view();
                // We're a frame late, but it should be fine.
                game_margin_top = editor_state.draw_editor_interface(
                    &mut platform,
                    &sdl,
                    &video_subsystem,
                    &game_window_events,
                    &mut painter,
                    target.as_ref(),
                );
                game_window_surface.end_frame(&gfx);
            }
        }

        // Nothing waits for the screen when no window is visible
        if !is_something_shown {
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}
