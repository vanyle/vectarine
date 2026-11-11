//! wgpu is built for EGL 1.5, but Emscripten only provides EGL 1.4. The two functions that are missing are never
//! called on the web: they are defined here so that the runtime links.

use std::ffi::c_void;

#[unsafe(no_mangle)]
pub extern "C" fn eglGetPlatformDisplay(
    _platform: u32,
    _native_display: *mut c_void,
    _attributes: *const isize,
) -> *mut c_void {
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn eglCreatePlatformWindowSurface(
    _display: *mut c_void,
    _config: *mut c_void,
    _native_window: *mut c_void,
    _attributes: *const isize,
) -> *mut c_void {
    std::ptr::null_mut()
}
