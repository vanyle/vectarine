use crate::game::Game;
use mlua::IntoLua;
use sdl2::{event::Event, keyboard::Keycode};
use std::collections::HashMap;

pub mod file;

#[derive(Clone, Debug, Default)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    pub is_left_down: bool,
    pub is_right_down: bool,
}

#[derive(Clone, Debug)]
pub struct IoEnvState {
    // Inputs
    pub window_width: u32,
    pub window_height: u32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub px_ratio_x: f32,
    pub px_ratio_y: f32,
    pub mouse_state: MouseState,
    pub keyboard_state: HashMap<Keycode, bool>,

    // Outputs
    pub is_window_resizeable: bool,
    pub fullscreen_state_request: Option<bool>,
    pub window_target_size: Option<(u32, u32)>,
}

impl Default for IoEnvState {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 600,
            screen_width: 0,
            screen_height: 0,
            px_ratio_x: 1.0,
            px_ratio_y: 1.0,
            mouse_state: MouseState::default(),
            keyboard_state: HashMap::new(),
            is_window_resizeable: false,
            window_target_size: None,
            fullscreen_state_request: None,
        }
    }
}

pub fn process_events(
    game: &mut Game,
    events: &[sdl2::event::Event],
    framebuffer_width: f32,
    framebuffer_height: f32,
) {
    for event in events.iter() {
        match event {
            Event::Quit { .. } => {
                std::process::exit(0);
            }
            Event::KeyUp { keycode, .. } => {
                let Some(keycode) = keycode else {
                    return;
                };
                let mut env_state = game.lua_env.env_state.borrow_mut();
                env_state.keyboard_state.insert(*keycode, false);

                let _ = game.lua_env.default_events.keyup_event.trigger(
                    &game.lua_env.lua,
                    keycode.name().into_lua(&game.lua_env.lua).unwrap(),
                );
            }
            Event::KeyDown { keycode, .. } => {
                let Some(keycode) = keycode else {
                    return;
                };
                let mut env_state = game.lua_env.env_state.borrow_mut();
                env_state.keyboard_state.insert(*keycode, true);

                let _ = game.lua_env.default_events.keydown_event.trigger(
                    &game.lua_env.lua,
                    keycode.name().into_lua(&game.lua_env.lua).unwrap(),
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
                    &game.lua_env.lua,
                    mouse_button.into_lua(&game.lua_env.lua).unwrap(),
                );
            }
            Event::MouseButtonDown { mouse_btn, .. } => {
                {
                    let mut env_state = game.lua_env.env_state.borrow_mut();
                    let mouse_state = &mut env_state.mouse_state;
                    if *mouse_btn == sdl2::mouse::MouseButton::Left {
                        mouse_state.is_left_down = true;
                    } else if *mouse_btn == sdl2::mouse::MouseButton::Right {
                        mouse_state.is_right_down = true;
                    }
                }
                let mouse_button = mouse_button_to_str(*mouse_btn);
                let _ = game.lua_env.default_events.mouse_down_event.trigger(
                    &game.lua_env.lua,
                    mouse_button.into_lua(&game.lua_env.lua).unwrap(),
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
