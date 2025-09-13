use std::collections::HashMap;

use sdl2::{event::Event, keyboard::Keycode};

use crate::helpers::game::Game;

#[derive(Clone, Debug, Default)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    pub is_left_down: bool,
    pub is_right_down: bool,
}

#[derive(Clone, Debug)]
pub struct IoEnvState {
    pub window_width: u32,
    pub window_height: u32,
    pub mouse_state: MouseState,
    pub keyboard_state: HashMap<Keycode, bool>,
}

impl Default for IoEnvState {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 600,
            mouse_state: MouseState::default(),
            keyboard_state: HashMap::new(),
        }
    }
}

pub fn process_events(game: &mut Game, events: &[sdl2::event::Event]) {
    for event in events.iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
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
                let mouse_state = &mut game.lua_env.env_state.borrow_mut().mouse_state;
                mouse_state.x = *x as f32;
                mouse_state.y = *y as f32;
                mouse_state.is_left_down = mousestate.left();
                mouse_state.is_right_down = mousestate.right();
            }
            _ => {}
        }
    }
}
