//! # Vectarine Plugin SDK
//!
//! The Vectarine Plugin SDK crate provide common functions to build your vectarine plugins.
//! Check the `editor-plugin-template` crate for a starting point.
//!
//! Internally, this crate is a dependency of the runtime and editor to make sure that both
//! use the same libraries with the same versions.

// Re-export commonly used crates for the editor
pub use anyhow;
pub use egui;
pub use egui_glow;
pub use lazy_static;
pub use mlua;
pub use rapier2d;
pub use serde;
pub use toml;

pub mod plugininterface;
