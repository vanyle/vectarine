use crate::drawing_surface::{DrawingSurface, SurfaceMargins};
use vectarine_plugin_sdk::glow::{self, HasContext};
use vectarine_plugin_sdk::sdl2::VideoSubsystem;
use vectarine_plugin_sdk::sdl2::sys::SDL_WindowFlags;
use vectarine_plugin_sdk::sdl2::video::{FullscreenType, Window, WindowPos};

pub struct SdlDrawingSurface {
    pub window: Window,
    pub video_subsystem: VideoSubsystem,

    // On Linux, drawable_size and size are the same which would make the window way to small.
    // We thus compute this scaling factor to increase the size of logical pixels.
    // This factor is used by egui to scale the UI and by vectarine's pixel functions.
    // On other platforms, this is 1.0
    scaling: (f32, f32),
}

impl DrawingSurface for SdlDrawingSurface {
    fn get_drawable_size_in_hardware_px(&self) -> (u32, u32) {
        drawable_screen_size(&self.window)
    }

    /// Returns the ratio of drawable (hardware) pixels to SDL pixels.
    fn density_ratio(&self) -> (f32, f32) {
        let (drawable_width, drawable_height) = self.get_drawable_size_in_hardware_px();
        let (window_width, window_height) = self.window.size();

        (
            drawable_width as f32 / window_width as f32,
            drawable_height as f32 / window_height as f32,
        )
    }

    /// Note that SDL provides no API to set the drawable size in hardware pixels, so we set the logical size instead.
    fn set_drawable_size_in_logical_px(&mut self, logical_width: u32, logical_height: u32) {
        let (scaling_x, scaling_y) = self.scaling;

        let _ = self.window.set_size(
            (logical_width as f32 * scaling_x) as u32,
            (logical_height as f32 * scaling_y) as u32,
        );

        #[cfg(target_os = "emscripten")]
        {
            use emscripten_val::Val;
            let _ = Val::global("vectarine").call("resizeCanvas", &[]);
        }
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
        let (drawable_width, drawable_height) = self.get_drawable_size_in_hardware_px();
        let (viewport_x, viewport_y, viewport_width, viewport_height) = (
            margins.left as i32,
            margins.top as i32,
            (drawable_width as f32 - margins.left - margins.right) as i32,
            (drawable_height as f32 - margins.top - margins.bottom) as i32,
        );
        unsafe { gl.viewport(viewport_x, viewport_y, viewport_width, viewport_height) }
    }

    fn convert_sdl_to_opengl_coordinates(
        &self,
        x: f32,
        y: f32,
        margins: &SurfaceMargins,
    ) -> (f32, f32) {
        let (width, height) = self.window.size();
        let (drawable_width, drawable_height) = self.get_drawable_size_in_hardware_px();

        // Convert margins from hardware pixels to logical pixels
        let (margin_left, margin_top, _margin_right, _margin_bottom) = (
            margins.left * width as f32 / drawable_width as f32,
            margins.top * height as f32 / drawable_height as f32,
            margins.right * width as f32 / drawable_width as f32,
            margins.bottom * height as f32 / drawable_height as f32,
        );

        (
            2.0 * (x - margin_left) / width as f32 - 1.0,
            -2.0 * (y - margin_top) / height as f32 + 1.0,
        )
    }

    fn get_size_in_vectarine_px(&self) -> (f32, f32) {
        let (scaling_x, scaling_y) = self.scaling;
        let (logical_width, logical_height) = self.window.size();
        (
            logical_width as f32 / scaling_x,
            logical_height as f32 / scaling_y,
        )
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

impl SdlDrawingSurface {
    pub fn new(
        window: Window,
        video_subsystem: VideoSubsystem,
        force_scaling: Option<(f32, f32)>,
    ) -> Self {
        // In screen-less environments, it makes sense to force a scaling of 1.0 to reliability.
        let scaling = if let Some(force_scaling) = force_scaling {
            force_scaling
        } else {
            get_scaling(&video_subsystem, &window)
        };
        Self {
            window,
            video_subsystem,
            scaling,
        }
    }
}

/// If the current OS is linux, return the screen size in mm.
#[allow(dead_code)]
fn get_screen_size_linux() -> Result<(u32, u32), String> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xrandr")
            .output()
            .map_err(|_| "xrandr command failed or not found".to_string())
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Look for "connected primary" or just "connected"
                for line in stdout.lines() {
                    // Look for a string like: "376mm x 301mm"
                    // The format is often like "... 1920x1080+0+0 (normal left inverted right x axis y axis) 527mm x 296mm"
                    if line.contains(" connected ")
                        && let Some(pos) = line.rfind(") ")
                    {
                        let size_str = &line[pos + 2..];
                        let parts: Vec<&str> = size_str.split("mm x ").collect();
                        if parts.len() == 2
                            && let (Ok(width), Ok(height)) = (
                                parts[0].trim().parse::<u32>(),
                                parts[1].trim_end_matches("mm").parse::<u32>(),
                            )
                        {
                            return Ok((width, height));
                        }
                    }
                }
                Err("Could not parse screen size from xrandr output".to_string())
            })
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("get_screen_size_linux is only supported on Linux".to_string())
    }
}

// Ratio between the drawable size and the window size.
fn get_scaling(_video_subsystem: &VideoSubsystem, _window: &Window) -> (f32, f32) {
    // On Ubuntu, windows are too small by default.
    // We resize the windows based on the total screen size so that they occupy roughly the same share of the screen as on other platforms.
    // To do so, we need to know the physical size of the screen in mm, and the size of the screen in pixels.
    #[cfg(target_os = "linux")]
    {
        // Use classic laptop size as a fallback
        let target_dpi = 96.0;
        let screen_size = get_screen_size_linux().unwrap_or((360, 240));
        let display_size_px = _video_subsystem
            .display_bounds(0)
            .ok()
            .map(|rect| (rect.w, rect.h))
            .unwrap_or((1920, 1080));
        let current_dpi_x = (display_size_px.0 as f32 / screen_size.0 as f32) * 25.4; // Convert mm to inches
        let current_dpi_y = (display_size_px.1 as f32 / screen_size.1 as f32) * 25.4;
        (
            f32::max(1.0, current_dpi_x / target_dpi),
            f32::max(1.0, current_dpi_y / target_dpi),
        )
    }

    // On other platforms, drawing_size and size are properly handled, so we don't need to do any scaling.
    #[cfg(not(target_os = "linux"))]
    {
        (1.0, 1.0)
    }
}
