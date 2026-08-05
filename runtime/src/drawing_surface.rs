use vectarine_plugin_sdk::{glow, sdl2::video::FullscreenType};

pub mod sdl_drawing_surface;

#[derive(Debug, Copy, Clone, Default)]
pub struct SurfaceMargins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

/// Represents a place on which the game is drawn.
/// This can be an SDL window directly, or a framebuffer.
pub trait DrawingSurface {
    /// Returns the size of the area that game can use to draw pixels on.
    /// A pixel is not a hardware pixel, but a logical pixel, just like in CSS.
    fn get_drawable_size_in_px(&self) -> (u32, u32);

    // Sets the size of the drawable area in pixels. This is the closest thing to "resizing the window".
    // On some platforms, this will just change the size of the framebuffer.
    fn set_drawable_size_in_px(&mut self, width: u32, height: u32);

    // Indicates that rendering should be paused/slowed because the surface is not visible.
    fn is_minimized(&self) -> bool;

    /// Requests the window to be centered.
    fn center_window(&mut self);

    fn is_fullscreen(&self) -> FullscreenType;
    fn set_is_fullscreen(&mut self, fullscreen: FullscreenType);

    // Can the surface be manually resized by the user? Depending on the platform, `set_resizable` might do nothing.
    fn is_resizable(&self) -> bool;
    fn set_resizable(&mut self, resizable: bool);

    fn get_screen_size_in_px(&self) -> (u32, u32);

    fn set_title(&mut self, title: &str);

    /// Wrapper around the OpenGL `glViewport` function to only draw in this area of the surface.
    /// # Safety
    /// The GL context provided needs to be valid.
    /// The caller needs to use the same thread that owns the GL context.
    unsafe fn configure_viewport(&self, gl: &glow::Context, margins: SurfaceMargins);
}
