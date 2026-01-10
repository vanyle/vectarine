#[cfg(target_os = "emscripten")]
pub mod emscripten;

#[cfg(not(target_os = "emscripten"))]
pub mod desktop;
