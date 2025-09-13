use std::{
    fs,
    path::Path,
    sync::{Arc, mpsc::channel},
    thread,
    time::{Duration, Instant},
};

use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use runtime::init_sdl;
use runtime::{
    graphics::batchdraw::BatchDraw2d,
    helpers::{
        game::Game,
        lua_env::{LuaEnvironment, run_file_and_display_error},
    },
};

fn main() {
    gui_main();
}

fn gui_main() {
    let (sdl, video, window, event_pump) = init_sdl();

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

    thread::spawn(move || {
        loop {
            let event = debounce_receiver.recv();
            if let Ok(event) = event {
                for path in event.event.paths {
                    if path.extension().is_some() && path.extension().unwrap() == "lua" {
                        // println!("Reloading script: {}", path.to_string_lossy());
                        let content = fs::read(&path);
                        let Ok(content) = content else {
                            println!("Failed to read file: {}", path.to_string_lossy());
                            continue;
                        };
                        run_file_and_display_error(&lua_for_reload, &content, &path);
                    }
                }
            }
        }
    });

    debouncer
        .watch("./assets", RecursiveMode::Recursive)
        .unwrap();

    let _gl_context = window
        .borrow()
        .gl_create_context()
        .expect("Failed to create GL context");

    let gl = unsafe {
        egui_glow::painter::Context::from_loader_function(|name| {
            video.borrow().gl_get_proc_address(name) as *const _
        })
    };

    let gl: Arc<glow::Context> = Arc::new(gl);

    let mut painter = egui_glow::Painter::new(gl.clone(), "", None, true).unwrap();

    // Create the egui + sdl2 platform
    let mut platform = egui_sdl2_platform::Platform::new(window.borrow().size()).unwrap();

    // Get the time before the loop started
    let start_time = Instant::now();
    let mut text_command = String::new();

    let batch = BatchDraw2d::new(&gl).unwrap();
    let mut game = Game::new(batch, event_pump, lua_env);

    // The main loop
    loop {
        // Update the time
        platform.update_time(start_time.elapsed().as_secs_f64());

        let latest_events = game.event_pump.poll_iter().collect::<Vec<_>>();

        // Render the game
        game.main_loop(&latest_events);

        // Get the egui context and begin drawing the frame
        let ctx = platform.context();
        // Draw an egui window
        egui::Window::new("Console").show(&ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("console")
                .max_height(300.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let messages = &mut game.lua_env.messages.lock().unwrap();
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
                    let messages = &mut game.lua_env.frame_messages.lock().unwrap();
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
