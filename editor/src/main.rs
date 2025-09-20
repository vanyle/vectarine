use std::{
    path::Path,
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use runtime::{RenderingBlock, helpers::game_resource::script_resource::ScriptResource, init_sdl};
use runtime::{
    graphics::batchdraw::BatchDraw2d,
    helpers::{game::Game, lua_env::LuaEnvironment},
};

use crate::{editorinterface::EditorState, reload::reload_assets_if_needed};

pub mod editorinterface;
pub mod reload;

fn main() {
    gui_main();
}

fn gui_main() {
    let RenderingBlock {
        sdl,
        video,
        window,
        event_pump,
        gl,
    } = init_sdl();

    // window.borrow_mut().set_bordered(false);

    let (debounce_event_sender, debounce_receiver) = channel();
    let mut debouncer = new_debouncer(
        Duration::from_millis(10),
        None,
        move |result: DebounceEventResult| match result {
            Ok(events) => events.iter().for_each(|event| {
                let _ = debounce_event_sender.send(event.clone());
            }),
            Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
        },
    )
    .unwrap();

    let lua_env = LuaEnvironment::new();

    let path = Path::new("scripts/game.lua");
    lua_env.resources.load_resource::<ScriptResource>(path);
    let lua_env_for_reload = lua_env.clone();

    debouncer
        .watch("./assets", RecursiveMode::Recursive)
        .unwrap();

    let mut painter = egui_glow::Painter::new(gl.clone(), "", None, true).unwrap();

    // Create the egui + sdl2 platform
    let mut platform = egui_sdl2_platform::Platform::new(window.borrow().drawable_size()).unwrap();

    let batch = BatchDraw2d::new(&gl).unwrap();
    let mut game = Game::new(&gl, batch, event_pump, lua_env);
    let mut editor_state = EditorState::new(video.clone(), window.clone(), gl.clone());
    editor_state.load_config();

    window.borrow_mut().set_resizable(true);

    game.load(&video, &window);

    // The main loop
    let mut start_of_frame = Instant::now();
    loop {
        let latest_events = game.event_pump.poll_iter().collect::<Vec<_>>();
        game.load_resource_as_needed(gl.clone());
        reload_assets_if_needed(
            &gl,
            &game.lua_env.resources,
            &lua_env_for_reload,
            &debounce_receiver,
        );

        // Render the game
        let new_start_of_frame = Instant::now();
        game.main_loop(&latest_events, &window, start_of_frame.elapsed(), true);
        start_of_frame = new_start_of_frame;
        editor_state.draw_editor_interface(
            &mut platform,
            &sdl,
            &mut game,
            &latest_events,
            &mut painter,
        );

        window.borrow().gl_swap_window();
    }
}
