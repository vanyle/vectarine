use std::{
    fs,
    path::Path,
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use runtime::{RenderingBlock, init_sdl};
use runtime::{
    graphics::batchdraw::BatchDraw2d,
    helpers::{
        game::Game,
        lua_env::{LuaEnvironment, run_file_and_display_error},
    },
};

use crate::reload::reload_assets_if_needed;

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

    let lua_for_reload = lua_env.clone();

    let path = Path::new("assets/scripts/game.lua");
    let content = fs::read(path);
    if let Ok(content) = content {
        run_file_and_display_error(&lua_for_reload, &content, path);
    }

    debouncer
        .watch("./assets", RecursiveMode::Recursive)
        .unwrap();

    let mut painter = egui_glow::Painter::new(gl.clone(), "", None, true).unwrap();

    // Create the egui + sdl2 platform
    let mut platform = egui_sdl2_platform::Platform::new(window.borrow().size()).unwrap();

    // Get the time before the loop started
    let start_time = Instant::now();
    let mut text_command = String::new();
    let mut is_console_shown = false;

    let batch = BatchDraw2d::new(&gl).unwrap();
    let mut game = Game::new(batch, event_pump, lua_env);

    game.load();

    // The main loop
    let mut now = Instant::now();
    loop {
        reload_assets_if_needed(&lua_for_reload, &debounce_receiver);
        // Update the time
        platform.update_time(start_time.elapsed().as_secs_f64());

        let latest_events = game.event_pump.poll_iter().collect::<Vec<_>>();

        // Render the game
        game.main_loop(&latest_events, &window, now.elapsed());
        now = Instant::now();

        // Get the egui context and begin drawing the frame
        let ctx = platform.context();

        if ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::NONE,
                egui::Key::I,
            )
        }) {
            is_console_shown = !is_console_shown;
        }
        egui::TopBottomPanel::top("toppanel").show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Vectarine Editor");
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Toggle console (Ctrl+Shift+I)").clicked() {
                            is_console_shown = !is_console_shown;
                        }
                    });
                });
            });
        });

        if is_console_shown {
            egui::Window::new("Console").show(&ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("console")
                    .max_height(300.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let messages = &mut game.lua_env.messages.borrow_mut();
                        for line in messages.iter().rev() {
                            ui.label(line);
                        }
                        messages.truncate(100);
                    });
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("frame console")
                    .max_height(300.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let messages = &mut game.lua_env.frame_messages.borrow_mut();
                        for line in messages.iter() {
                            ui.label(line);
                        }
                        messages.clear();
                    });
                ui.separator();
                let response = ui.add(egui::TextEdit::singleline(&mut text_command));
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // println!("Running command: {text_command}");
                    text_command.clear();
                    response.request_focus();
                }
            });
        }

        // Stop drawing the egui frame and get the full output
        let full_output = platform.end_frame(&mut video.borrow_mut()).unwrap();
        // Get the paint jobs
        let paint_jobs = platform.tessellate(&full_output);
        let pj = paint_jobs.as_slice();

        // Render the editor interface on top of the game.
        let size = window.borrow().size();
        painter.paint_and_update_textures([size.0, size.1], 1.0, pj, &full_output.textures_delta);
        window.borrow().gl_swap_window();

        for event in latest_events {
            platform.handle_event(&event, &sdl, &video.borrow());
        }
    }
}
