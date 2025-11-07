//! An graphics-backend independant egui backend for sdl2
pub mod conversions;
pub mod platform;

pub use conversions::*;
pub use platform::*;

/// SDL2 is re-exported to enable easier version sync for users
pub use runtime::sdl2;
