use std::{sync::mpsc::channel, time::Instant};

use egui_sdl2_platform::sdl2::event::{Event, WindowEvent};
use runtime::{RenderingBlock, game::drawable_screen_size, init_sdl};

use crate::{
    editorinterface::{EditorState, process_events_when_no_game},
    reload::reload_assets_if_needed,
};

pub mod editorconsole;
pub mod editorinterface;
pub mod editormenu;
pub mod editorresources;
pub mod editorwatcher;
pub mod projectstate;
pub mod reload;

fn main() {
    gui_main();
}

fn gui_main() {
    let RenderingBlock {
        sdl,
        video,
        window,
        mut event_pump,
        gl,
    } = init_sdl();

    let (debounce_event_sender, debounce_receiver) = channel();

    // window.borrow_mut().set_bordered(false);

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
    editor_state.load_config(true);
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
    let mut start_of_frame = Instant::now();
    loop {
        let latest_events = event_pump.poll_iter().collect::<Vec<_>>();

        let new_start_of_frame = Instant::now();
        if let Some(project) = editor_state.project.borrow_mut().as_mut() {
            let game = &mut project.game;

            game.load_resource_as_needed(gl.clone());
            reload_assets_if_needed(
                &gl,
                &game.lua_env.resources,
                &game.lua_env,
                &debounce_receiver,
            );

            // Render the game
            game.main_loop(&latest_events, &window, start_of_frame.elapsed(), true);
        } else {
            // Clear the screen when no project is loaded
            process_events_when_no_game(&latest_events, &gl);
        }
        start_of_frame = new_start_of_frame;

        editor_state.draw_editor_interface(&mut platform, &sdl, &latest_events, &mut painter);
        window.borrow().gl_swap_window();
    }
}
