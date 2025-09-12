use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use glow::HasContext;
use runtime::helpers::lua_env::{self, run_file_and_display_error};
use runtime::init_sdl;
use sdl2::event::{Event, WindowEvent};

pub fn main() {
    unsafe {
        use runtime::helpers::{file::read_file, game::Game};

        let (sdl, video, window, mut event_pump) = init_sdl();
        let lua_env = lua_env::LuaEnvironment::new();

        //let canvas = window.into_canvas().build().unwrap();
        let start_time = Instant::now();
        let mut color = [0.0, 0.0, 0.0];
        let mut text = String::new();

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

        let vertex_array = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vertex_array));

        let program = gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"out vec2 vert;
            void main() {
                vec2 verts[3];
                verts[0] = vec2(0.5, 1.0);
                verts[1] = vec2(0.0, 0.0);
                verts[2] = vec2(1.0, 0.0);

                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
            r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());
        let shader_version = "#version 300 es";

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{shader_version}\n{shader_source}"));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program));

        read_file(
            "game.lua",
            Box::new(move |content| {
                run_file_and_display_error(&lua_env, content.as_bytes(), Path::new("game.lua"));

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
                    let full_output = platform.end_frame(&mut video.borrow_mut()).unwrap();
                    // Get the paint jobs
                    let paint_jobs = platform.tessellate(&full_output);
                    let pj = paint_jobs.as_slice();

                    unsafe {
                        // We have opengl setup in this gl object.
                        // We can use it to setup glium!
                        gl.clear_color(color[0], color[1], color[2], 1.0);
                        gl.clear(glow::COLOR_BUFFER_BIT);

                        // We can do more opengl rendering here.
                        gl.use_program(Some(program));
                        gl.bind_vertex_array(Some(vertex_array));
                        painter.gl().draw_arrays(glow::TRIANGLES, 0, 3);
                    }

                    let size = window.borrow().size();

                    painter.paint_and_update_textures(
                        [size.0, size.1],
                        1.0,
                        pj,
                        &full_output.textures_delta,
                    );

                    window.borrow().gl_swap_window();

                    // Handle sdl events
                    for event in event_pump.poll_iter() {
                        // Handle sdl events
                        match event {
                            Event::Window {
                                window_id,
                                win_event,
                                ..
                            } => {
                                if window_id == window.borrow().id() {
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
                        platform.handle_event(&event, &sdl, &video.borrow());
                    }
                });
            }),
        );

        // Prevent exit from destroying the GL context.
        #[cfg(target_os = "emscripten")]
        {
            emscripten_functions::emscripten::exit_with_live_runtime();
        }
    }
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
