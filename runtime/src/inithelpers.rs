use std::cell::RefCell;
use std::rc::Rc;

use sdl2::{EventPump, Sdl};
use vectarine_plugin_sdk::sdl2;

use crate::drawing_surface::sdl_drawing_surface::SdlDrawingSurface;
use crate::gamepad::GamepadSystem;
use crate::graphics::gfx::Gfx;
use crate::graphics::surface::WindowSurface;

/// A datastructure that holds the primitives needed to interact with the environment. (windows, graphics, io, sound, etc.)
pub struct RenderingBlock {
    /// What is shown in `window`. It must not outlive the window.
    pub surface: WindowSurface,
    pub gfx: Gfx,
    pub window: Rc<RefCell<SdlDrawingSurface>>,
    pub event_pump: EventPump,
    pub sdl: Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub gamepad_system: Rc<dyn GamepadSystem>,
}
