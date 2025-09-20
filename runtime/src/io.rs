use crate::game::Game;
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
            }
            Event::KeyDown { keycode, .. } => {
                let Some(keycode) = keycode else {
                    return;
                };
                let mut env_state = game.lua_env.env_state.borrow_mut();
                env_state.keyboard_state.insert(*keycode, true);
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
