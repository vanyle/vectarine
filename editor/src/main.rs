use std::{
    fs,
    path::Path,
    sync::{Arc, mpsc::channel},
    thread,
    time::{Duration, Instant},
};

use egui_sdl2_platform::sdl2::{
    self,
    event::{Event, WindowEvent},
};
use glow::HasContext;
use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use runtime::helpers::{
    game::Game,
    lua_env::{LuaEnvironment, run_file_and_display_error},
};
use runtime::init_sdl;

fn main() {
    gui_main();
}

fn gui_main() {
    let (sdl, mut video, window, mut event_pump) = init_sdl();

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

    let path = Path::new("game.lua");
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

    debouncer.watch(".", RecursiveMode::NonRecursive).unwrap();

    let _gl_context = window
        .gl_create_context()
        .expect("Failed to create GL context");

    let gl = unsafe {
        egui_glow::painter::Context::from_loader_function(|name| {
            video.gl_get_proc_address(name) as *const _
        })
    };
    let mut painter = egui_glow::Painter::new(Arc::new(gl), "", None, true).unwrap();

    // Create the egui + sdl2 platform
    let mut platform = egui_sdl2_platform::Platform::new(window.size()).unwrap();

    // The clear color
    let mut color = [0.0, 0.0, 0.0, 1.0];
    // The textedit text
    let mut text = String::new();

    // Get the time before the loop started
    let start_time = Instant::now();

    // The main loop
    'main: loop {
        // Update the time
        platform.update_time(start_time.elapsed().as_secs_f64());

        // Get the egui context and begin drawing the frame
        let ctx = platform.context();
        // Draw an egui window
        egui::Window::new("Hello, world!").show(&ctx, |ui| {
            ui.label("Hello, world!");
            if ui.button("Greet").clicked() {
                println!("Hello, world!");
            }
            ui.horizontal(|ui| {
                ui.label("Color: ");
                ui.color_edit_button_rgba_premultiplied(&mut color);
            });
            ui.code_editor(&mut text);
        });

        // Stop drawing the egui frame and get the full output
        let full_output = platform.end_frame(&mut video).unwrap();
        // Get the paint jobs
        let paint_jobs = platform.tessellate(&full_output);
        let pj = paint_jobs.as_slice();

        unsafe {
            painter.gl().clear_color(color[0], color[1], color[2], 1.0);
            painter.gl().clear(glow::COLOR_BUFFER_BIT);
        }

        let size = window.size();
        painter.paint_and_update_textures([size.0, size.1], 1.0, pj, &full_output.textures_delta);
        window.gl_swap_window();

        // Handle sdl events
        for event in event_pump.poll_iter() {
            // Handle sdl events
            match event {
                Event::Window {
                    window_id,
                    win_event,
                    ..
                } => {
                    if window_id == window.id() {
                        if let WindowEvent::Close = win_event {
                            break 'main;
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
            // Let the egui platform handle the event
            platform.handle_event(&event, &sdl, &video);
        }
    }
}
