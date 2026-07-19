use crate::{game::Game, lua_env::print_lua_error_from_error};
use std::collections::HashMap;
use vectarine_plugin_sdk::mlua::IntoLua;
use vectarine_plugin_sdk::sdl2::{self, event::Event, keyboard::Scancode, video::FullscreenType};

pub mod dummyfs;
pub mod fs;
pub mod localfs;
pub mod time;
pub mod zipfs;

#[derive(Clone, Debug, Default)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    pub wheel_x: f32,
    pub wheel_y: f32,
    pub is_left_down: bool,
    pub is_right_down: bool,
    pub is_left_just_pressed: bool,
    pub is_right_just_pressed: bool,
}

#[derive(Clone, Debug)]
pub struct TouchState {
    pub id: i64,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
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
    pub current_touches: HashMap<(i64, i64), TouchState>,
    pub keyboard_state: HashMap<Scancode, bool>,
    pub keyboard_just_pressed_state: HashMap<Scancode, bool>,
    // The text typed since the last frame.
    pub text_input: String,

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
            current_touches: HashMap::new(),
            keyboard_state: HashMap::new(),
            keyboard_just_pressed_state: HashMap::new(),
            text_input: String::new(),

            start_time: std::time::Instant::now(),

            is_window_resizeable: false,
            window_target_size: None,
            fullscreen_state_request: None,
            center_window_request: false,
            window_title: None,
        }
    }
}

pub fn process_events<'a>(
    game: &mut Game,
    events: impl Iterator<Item = &'a sdl2::event::Event>,
    framebuffer_width: f32,
    framebuffer_height: f32,
) {
    {
        let mut env_state = game.lua_env.env_state.borrow_mut();
        env_state.keyboard_just_pressed_state.clear();
        env_state.mouse_state.is_left_just_pressed = false;
        env_state.mouse_state.is_right_just_pressed = false;
        env_state.mouse_state.wheel_x = 0.0;
        env_state.mouse_state.wheel_y = 0.0;
        env_state.text_input.clear();
    }

    for event in events {
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

                let lua_res = game.lua_env.default_events.keyup_event.trigger(
                    scancode
                        .name()
                        .into_lua(&game.lua_env.lua_handle.lua)
                        .expect("Failed to convert Keycode to Lua"),
                );
                if let Err(err) = lua_res {
                    print_lua_error_from_error(&game.lua_env.lua_handle, &err);
                }
            }
            Event::KeyDown { scancode, .. } => {
                let Some(scancode) = scancode else {
                    return;
                };
                let mut env_state = game.lua_env.env_state.borrow_mut();
                // The key is not just pressed if the keyboard_state contains true.
                if env_state.keyboard_state.get(scancode).copied() != Some(true) {
                    env_state
                        .keyboard_just_pressed_state
                        .insert(*scancode, true);
                }

                env_state.keyboard_state.insert(*scancode, true);

                let lua_res = game.lua_env.default_events.keydown_event.trigger(
                    scancode
                        .name()
                        .into_lua(&game.lua_env.lua_handle.lua)
                        .expect("Failed to convert Keycode to Lua"),
                );
                if let Err(err) = lua_res {
                    print_lua_error_from_error(&game.lua_env.lua_handle, &err);
                }
            }
            Event::TextInput { text, .. } => {
                let lua = &game.lua_env.lua_handle.lua;
                {
                    let mut env_state = game.lua_env.env_state.borrow_mut();
                    env_state.text_input.push_str(text);
                }
                let lua_res = game.lua_env.default_events.text_input_event.trigger(
                    text.clone()
                        .into_lua(lua)
                        .expect("Failed to convert text to Lua"),
                );
                if let Err(err) = lua_res {
                    print_lua_error_from_error(&game.lua_env.lua_handle, &err);
                }
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
                let lua_res = game.lua_env.default_events.mouse_up_event.trigger(
                    mouse_button
                        .into_lua(&game.lua_env.lua_handle.lua)
                        .expect("Failed to convert mouse button to Lua"),
                );
                if let Err(err) = lua_res {
                    print_lua_error_from_error(&game.lua_env.lua_handle, &err);
                }
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
                let lua_res = game.lua_env.default_events.mouse_down_event.trigger(
                    mouse_button
                        .into_lua(&game.lua_env.lua_handle.lua)
                        .expect("Failed to convert mouse button to Lua"),
                );
                if let Err(err) = lua_res {
                    print_lua_error_from_error(&game.lua_env.lua_handle, &err);
                }
            }
            Event::MouseWheel {
                timestamp: _,
                window_id: _,
                which: _,
                x: _,
                y: _,
                direction: _,
                precise_x,
                precise_y,
                mouse_x: _,
                mouse_y: _,
            } => {
                let mut env_state = game.lua_env.env_state.borrow_mut();
                let mouse_state = &mut env_state.mouse_state;
                mouse_state.wheel_x += *precise_x;
                mouse_state.wheel_y += *precise_y;
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
            Event::FingerDown {
                touch_id,
                finger_id,
                x,
                y,
                pressure,
                ..
            }
            | Event::FingerMotion {
                touch_id,
                finger_id,
                x,
                y,
                pressure,
                ..
            } => {
                let mut env_state = game.lua_env.env_state.borrow_mut();
                update_touch(&mut env_state, *touch_id, *finger_id, *x, *y, *pressure);
            }
            Event::FingerUp {
                touch_id,
                finger_id,
                ..
            } => {
                remove_touch(
                    &mut game.lua_env.env_state.borrow_mut(),
                    *touch_id,
                    *finger_id,
                );
            }
            _ => {}
        }
    }
}

fn update_touch(
    env_state: &mut IoEnvState,
    touch_id: i64,
    finger_id: i64,
    x: f32,
    y: f32,
    pressure: f32,
) {
    env_state.current_touches.insert(
        (touch_id, finger_id),
        TouchState {
            id: finger_id,
            x: x * 2.0 - 1.0,
            y: 1.0 - y * 2.0,
            pressure,
        },
    );
}

fn remove_touch(env_state: &mut IoEnvState, touch_id: i64, finger_id: i64) {
    env_state.current_touches.remove(&(touch_id, finger_id));
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

#[cfg(test)]
mod tests {
    use super::{IoEnvState, remove_touch, update_touch};

    #[test]
    fn touch_positions_use_opengl_coordinates() {
        let mut state = IoEnvState::default();

        update_touch(&mut state, 1, 10, 0.25, 0.75, 0.5);

        let touch = state
            .current_touches
            .get(&(1, 10))
            .expect("touch should be registered");
        assert_eq!(touch.id, 10);
        assert_eq!(touch.x, -0.5);
        assert_eq!(touch.y, -0.5);
        assert_eq!(touch.pressure, 0.5);
    }

    #[test]
    fn touch_updates_keep_fingers_independent() {
        let mut state = IoEnvState::default();

        update_touch(&mut state, 1, 10, 0.0, 0.0, 0.25);
        update_touch(&mut state, 1, 20, 1.0, 1.0, 0.75);
        update_touch(&mut state, 1, 10, 0.5, 0.5, 1.0);

        assert_eq!(state.current_touches.len(), 2);
        let first_touch = state
            .current_touches
            .get(&(1, 10))
            .expect("first touch should be registered");
        assert_eq!(first_touch.x, 0.0);
        assert_eq!(first_touch.y, 0.0);
        assert_eq!(first_touch.pressure, 1.0);
        assert_eq!(
            state
                .current_touches
                .get(&(1, 20))
                .expect("second touch should be registered")
                .id,
            20
        );

        remove_touch(&mut state, 1, 10);

        assert_eq!(state.current_touches.len(), 1);
        assert!(!state.current_touches.contains_key(&(1, 10)));
        assert!(state.current_touches.contains_key(&(1, 20)));
    }
}
