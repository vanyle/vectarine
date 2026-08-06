use crate::drawing_surface::{DrawingSurface, SurfaceMargins};
use vectarine_plugin_sdk::glow::{self, HasContext};
use vectarine_plugin_sdk::sdl2::VideoSubsystem;
use vectarine_plugin_sdk::sdl2::sys::SDL_WindowFlags;
use vectarine_plugin_sdk::sdl2::video::{FullscreenType, Window, WindowPos};

pub struct SdlDrawingSurface {
    pub window: Window,
    pub video_subsystem: VideoSubsystem,
}

impl DrawingSurface for SdlDrawingSurface {
    fn get_drawable_size_in_px(&self) -> (u32, u32) {
        // Note: we use size, not drawable size here! drawable size is hidden from the game maker, so that pixels have roughly the same size on all platforms
        drawable_screen_size(&self.window)
    }

    fn set_drawable_size_in_px(&mut self, logical_width: u32, logical_height: u32) {
        let scaling = get_linux_window_scaling(&self.video_subsystem);
        let _ = self.window.set_size(
            (logical_width as f32 * scaling) as u32,
            (logical_height as f32 * scaling) as u32,
        );
    }

    fn is_minimized(&self) -> bool {
        self.window.is_minimized()
    }

    fn center_window(&mut self) {
        self.window
            .set_position(WindowPos::Centered, WindowPos::Centered);
    }

    fn is_fullscreen(&self) -> FullscreenType {
        self.window.fullscreen_state()
    }

    fn set_is_fullscreen(&mut self, fullscreen: FullscreenType) {
        // Discard errors on failure
        let _ = self.window.set_fullscreen(fullscreen);
    }

    fn is_resizable(&self) -> bool {
        let flags = self.window.window_flags();
        (flags & (SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32)) != 0
    }

    fn set_resizable(&mut self, resizable: bool) {
        self.window.set_resizable(resizable);
    }

    fn get_screen_size_in_px(&self) -> (u32, u32) {
        get_screen_size(&self.window)
    }

    fn set_title(&mut self, title: &str) {
        // SDL's set_title works on the web by default
        let _ = self.window.set_title(title);
    }

    unsafe fn configure_viewport(&self, gl: &glow::Context, margins: SurfaceMargins) {
        let (drawable_width, drawable_height) = self.get_drawable_size_in_px();
        let (viewport_x, viewport_y, viewport_width, viewport_height) = (
            margins.left as i32,
            margins.top as i32,
            (drawable_width as f32 - margins.left - margins.right) as i32,
            (drawable_height as f32 - margins.top - margins.bottom) as i32,
        );
        unsafe { gl.viewport(viewport_x, viewport_y, viewport_width, viewport_height) }
    }
}

// Read index.html for the JS implementation of the window functions.

#[cfg(not(target_os = "emscripten"))]
pub fn drawable_screen_size(window: &Window) -> (u32, u32) {
    window.drawable_size()
}

#[cfg(target_os = "emscripten")]
pub fn drawable_screen_size(_window: &Window) -> (u32, u32) {
    use emscripten_val::Val;
    // On the web, the drawable size and the screen size are the same.
    // Aspect ratio is preserved at the JS level, not here.
    let size = Val::global("vectarine").call("getDrawableScreenSize", &[]);
    let width = size.get(&Val::from_str("width")).as_i32();
    let height = size.get(&Val::from_str("height")).as_i32();
    (width as u32, height as u32)
}

#[cfg(not(target_os = "emscripten"))]
pub fn set_drawable_screen_size(window: &mut Window, width: u32, height: u32) {
    let _ = window.set_size(width, height);
}

#[cfg(target_os = "emscripten")]
pub fn set_drawable_screen_size(_window: &mut Window, width: u32, height: u32) {
    use emscripten_val::Val;
    // Resize the underlying canvas.
    let _ = Val::global("vectarine").call(
        "setDrawableScreenSize",
        &[&Val::from_(width as f64), &Val::from_(height as f64)],
    );
}

#[cfg(not(target_os = "emscripten"))]
pub fn get_screen_size(window: &Window) -> (u32, u32) {
    let display = window.subsystem().current_display_mode(0);
    if let Ok(display) = display {
        (display.w as u32, display.h as u32)
    } else {
        window.size()
    }
}

#[cfg(target_os = "emscripten")]
pub fn get_screen_size(_window: &Window) -> (u32, u32) {
    use emscripten_val::Val;
    // getScreenSize is the size of the html (as provided by clientWidth and clientHeight).
    // The canvas does it's best to fit in that size.
    let size = Val::global("vectarine").call("getScreenSize", &[]);
    let width = size.get(&Val::from_str("width")).as_i32();
    let height = size.get(&Val::from_str("height")).as_i32();
    (width as u32, height as u32)
}

/// On Ubuntu, windows are too small by default.
/// We resize the windows based on the total screen size so that they occupy roughly the same share of the screen as on other platforms.
pub fn get_linux_window_scaling(_video_subsystem: &VideoSubsystem) -> f32 {
    #[cfg(target_os = "linux")]
    {
        let display_width = _video_subsystem
            .display_bounds(0)
            .map(|rect| rect.width())
            .ok()
            .unwrap_or(1920);
        f32::max(1.0, display_width as f32 / 1920.0)
    }

    #[cfg(not(target_os = "linux"))]
    {
        1.0
    }
}
