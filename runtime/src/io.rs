use crate::game::Game;
use sdl2::{self, event::Event, keyboard::Scancode, video::FullscreenType};
use std::collections::HashMap;
use vectarine_plugin_sdk::mlua::IntoLua;

pub mod dummyfs;
pub mod fs;
pub mod localfs;
pub mod time;
pub mod zipfs;

#[derive(Clone, Debug, Default)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    pub is_left_down: bool,
    pub is_right_down: bool,
    pub is_left_just_pressed: bool,
    pub is_right_just_pressed: bool,
}

#[derive(Debug)]
pub struct IoEnvState {
    // Inputs
    pub window_width: u32,
    pub window_height: u32,
    pub is_window_minimized: bool,
    pub screen_width: u32,
    pub screen_height: u32,
    pub px_ratio_x: f32,
    pub px_ratio_y: f32,
    pub mouse_state: MouseState,
    pub keyboard_state: HashMap<Scancode, bool>,
    pub keyboard_just_pressed_state: HashMap<Scancode, bool>,

    pub start_time: std::time::Instant,

    // Outputs
    pub is_window_resizeable: bool,
    pub center_window_request: bool,
    pub fullscreen_state_request: Option<FullscreenType>,
    pub window_target_size: Option<(u32, u32)>,
    pub window_title: Option<String>,
}

impl Default for IoEnvState {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 600,
            screen_width: 0,
            screen_height: 0,
            is_window_minimized: false,
            px_ratio_x: 1.0,
            px_ratio_y: 1.0,
            mouse_state: MouseState::default(),
            keyboard_state: HashMap::new(),
            keyboard_just_pressed_state: HashMap::new(),

            start_time: std::time::Instant::now(),

            is_window_resizeable: false,
            window_target_size: None,
            fullscreen_state_request: None,
            center_window_request: false,
            window_title: None,
        }
    }
}

pub fn process_events(
    game: &mut Game,
    events: &[sdl2::event::Event],
    framebuffer_width: f32,
    framebuffer_height: f32,
) {
    {
        let mut env_state = game.lua_env.env_state.borrow_mut();
        env_state.keyboard_just_pressed_state.clear();
        env_state.mouse_state.is_left_just_pressed = false;
        env_state.mouse_state.is_right_just_pressed = false;
    }

    for event in events.iter() {
        match event {
            Event::Quit { .. } => {
                std::process::exit(0);
            }
            Event::KeyUp { scancode, .. } => {
                let Some(scancode) = scancode else {
                    return;
                };
                let mut env_state = game.lua_env.env_state.borrow_mut();
                env_state.keyboard_state.insert(*scancode, false);

                let _ = game.lua_env.default_events.keyup_event.trigger(
                    scancode
                        .name()
                        .into_lua(&game.lua_env.lua)
                        .expect("Failed to convert Keycode to Lua"),
                );
            }
            Event::KeyDown { scancode, .. } => {
                let Some(scancode) = scancode else {
                    return;
                };
                let mut env_state = game.lua_env.env_state.borrow_mut();
                env_state.keyboard_state.insert(*scancode, true);
                env_state
                    .keyboard_just_pressed_state
                    .insert(*scancode, true);

                let _ = game.lua_env.default_events.keydown_event.trigger(
                    scancode
                        .name()
                        .into_lua(&game.lua_env.lua)
                        .expect("Failed to convert Keycode to Lua"),
                );
            }
            Event::TextInput { text, .. } => {
                let lua = &game.lua_env.lua;
                let _ = game.lua_env.default_events.text_input_event.trigger(
                    text.clone()
                        .into_lua(lua)
                        .expect("Failed to convert text to Lua"),
                );
            }
            Event::MouseButtonUp { mouse_btn, .. } => {
                {
                    let mut env_state = game.lua_env.env_state.borrow_mut();
                    let mouse_state = &mut env_state.mouse_state;
                    if *mouse_btn == sdl2::mouse::MouseButton::Left {
                        mouse_state.is_left_down = false;
                    } else if *mouse_btn == sdl2::mouse::MouseButton::Right {
                        mouse_state.is_right_down = false;
                    }
                }
                let mouse_button = mouse_button_to_str(*mouse_btn);
                let _ = game.lua_env.default_events.mouse_up_event.trigger(
                    mouse_button
                        .into_lua(&game.lua_env.lua)
                        .expect("Failed to convert mouse button to Lua"),
                );
            }
            Event::MouseButtonDown { mouse_btn, .. } => {
                {
                    let mut env_state = game.lua_env.env_state.borrow_mut();
                    let mouse_state = &mut env_state.mouse_state;
                    if *mouse_btn == sdl2::mouse::MouseButton::Left {
                        mouse_state.is_left_down = true;
                        mouse_state.is_left_just_pressed = true;
                    } else if *mouse_btn == sdl2::mouse::MouseButton::Right {
                        mouse_state.is_right_down = true;
                        mouse_state.is_right_just_pressed = true;
                    }
                }
                let mouse_button = mouse_button_to_str(*mouse_btn);
                let _ = game.lua_env.default_events.mouse_down_event.trigger(
                    mouse_button
                        .into_lua(&game.lua_env.lua)
                        .expect("Failed to convert mouse button to Lua"),
                );
            }
            Event::MouseMotion {
                timestamp: _,
                window_id: _,
                which: _,
                mousestate,
                x,
                y,
                xrel: _,
                yrel: _,
            } => {
                let mut env_state = game.lua_env.env_state.borrow_mut();
                let px_ratio_x = env_state.px_ratio_x; // convert between real and fake pixels
                let px_ratio_y = env_state.px_ratio_y;
                let mouse_state = &mut env_state.mouse_state;

                mouse_state.x = (*x as f32) * px_ratio_x / framebuffer_width * 2.0 - 1.0;
                mouse_state.y = -((*y as f32) * px_ratio_y / framebuffer_height * 2.0 - 1.0);
                mouse_state.is_left_down = mousestate.left();
                mouse_state.is_right_down = mousestate.right();
            }
            _ => {}
        }
    }
}

fn mouse_button_to_str(mouse_btn: sdl2::mouse::MouseButton) -> &'static str {
    if mouse_btn == sdl2::mouse::MouseButton::Left {
        "left"
    } else if mouse_btn == sdl2::mouse::MouseButton::Right {
        "right"
    } else if mouse_btn == sdl2::mouse::MouseButton::Middle {
        "middle"
    } else {
        "???"
    }
}
