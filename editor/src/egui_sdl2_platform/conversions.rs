use egui::Key;
use runtime::sdl2::keyboard::Keycode;

/// A trait that adds a method to convert to an egui key
pub trait ToEguiKey {
    /// Convert the struct to an egui key
    fn to_egui_key(&self) -> Option<egui::Key>;
}

impl ToEguiKey for runtime::sdl2::keyboard::Keycode {
    fn to_egui_key(&self) -> Option<egui::Key> {
        Some(match *self {
            Keycode::Left => Key::ArrowLeft,
            Keycode::Up => Key::ArrowUp,
            Keycode::Right => Key::ArrowRight,
            Keycode::Down => Key::ArrowDown,
            Keycode::Escape => Key::Escape,
            Keycode::Tab => Key::Tab,
            Keycode::Backspace => Key::Backspace,
            Keycode::Space => Key::Space,
            Keycode::Return => Key::Enter,
            Keycode::Insert => Key::Insert,
            Keycode::Home => Key::Home,
            Keycode::Delete => Key::Delete,
            Keycode::End => Key::End,
            Keycode::PageDown => Key::PageDown,
            Keycode::PageUp => Key::PageUp,
            Keycode::Kp0 | Keycode::Num0 => Key::Num0,
            Keycode::Kp1 | Keycode::Num1 => Key::Num1,
            Keycode::Kp2 | Keycode::Num2 => Key::Num2,
            Keycode::Kp3 | Keycode::Num3 => Key::Num3,
            Keycode::Kp4 | Keycode::Num4 => Key::Num4,
            Keycode::Kp5 | Keycode::Num5 => Key::Num5,
            Keycode::Kp6 | Keycode::Num6 => Key::Num6,
            Keycode::Kp7 | Keycode::Num7 => Key::Num7,
            Keycode::Kp8 | Keycode::Num8 => Key::Num8,
            Keycode::Kp9 | Keycode::Num9 => Key::Num9,
            Keycode::A => Key::A,
            Keycode::B => Key::B,
            Keycode::C => Key::C,
            Keycode::D => Key::D,
            Keycode::E => Key::E,
            Keycode::F => Key::F,
            Keycode::G => Key::G,
            Keycode::H => Key::H,
            Keycode::I => Key::I,
            Keycode::J => Key::J,
            Keycode::K => Key::K,
            Keycode::L => Key::L,
            Keycode::M => Key::M,
            Keycode::N => Key::N,
            Keycode::O => Key::O,
            Keycode::P => Key::P,
            Keycode::Q => Key::Q,
            Keycode::R => Key::R,
            Keycode::S => Key::S,
            Keycode::T => Key::T,
            Keycode::U => Key::U,
            Keycode::V => Key::V,
            Keycode::W => Key::W,
            Keycode::X => Key::X,
            Keycode::Y => Key::Y,
            Keycode::Z => Key::Z,
            _ => {
                return None;
            }
        })
    }
}
