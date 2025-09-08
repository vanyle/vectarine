use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use glow::HasContext;
use runtime::helpers::lua_env::{self, run_file_and_display_error};
use runtime::init_sdl;
use sdl2::event::{Event, WindowEvent};

pub fn main() {
    use runtime::helpers::{file::read_file, game::Game};

    let (sdl, mut video, window, mut event_pump) = init_sdl();
    let lua_env = lua_env::LuaEnvironment::new();

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

    //let canvas = window.into_canvas().build().unwrap();
    let start_time = Instant::now();
    let mut color = [0.0, 0.0, 0.0];
    let mut text = String::new();

    // read_file(
    //     "game.lua",
    //     Box::new(move |content| {
    //         run_file_and_display_error(&lua_env, content.as_bytes(), Path::new("game.lua"));

    // let mut game = Game::new(canvas, event_pump, lua_env);
    // set_main_loop stops the current function being executed.
    set_main_loop_wrapper(move || {
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
                ui.add(egui::Slider::new(&mut color[0], 0.0..=1.0).text("R"));
                ui.add(egui::Slider::new(&mut color[1], 0.0..=1.0).text("G"));
                ui.add(egui::Slider::new(&mut color[2], 0.0..=1.0).text("B"));
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
                            std::process::exit(0);
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => {
                    std::process::exit(0);
                }
                _ => {}
            }
            // Let the egui platform handle the event
            platform.handle_event(&event, &sdl, &video);
        }
    });
    //     }),
    // );
}

#[allow(unused_mut)]
fn set_main_loop_wrapper<F>(mut f: F)
where
    F: FnMut() + 'static,
{
    #[cfg(target_os = "emscripten")]
    {
        emscripten_functions::emscripten::set_main_loop(f, 0, true);
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        // TODO: true vsync
        loop {
            f();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}
