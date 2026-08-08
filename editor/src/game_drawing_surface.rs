use std::{cell::RefCell, rc::Rc};

use runtime::{
    drawing_surface::{DrawingSurface, SurfaceMargins},
    glow::{self, HasContext},
};

pub struct GameDrawingSurface {
    // Size of the fake window inside the editor
    pub target_size: (u32, u32),
    pub resizable: bool,
    pub title: String,

    // True surface of the editor
    pub underlying_window: Rc<RefCell<dyn DrawingSurface>>,
}

impl GameDrawingSurface {
    pub fn new(underlying_window: Rc<RefCell<dyn DrawingSurface>>) -> Self {
        let (width, height) = underlying_window
            .borrow()
            .get_drawable_size_in_hardware_px();
        Self {
            target_size: (width, height),
            resizable: true,
            title: "Game".to_string(),
            underlying_window,
        }
    }
}

impl DrawingSurface for GameDrawingSurface {
    fn get_drawable_size_in_hardware_px(&self) -> (u32, u32) {
        self.target_size
    }

    fn density_ratio(&self) -> (f32, f32) {
        // The density does not depend on what the game surface is set to, but on the underlying window.
        self.underlying_window.borrow().density_ratio()
    }

    fn set_drawable_size_in_logical_px(&mut self, width: u32, height: u32) {
        self.target_size = (width, height);
    }

    fn is_minimized(&self) -> bool {
        self.underlying_window.borrow().is_minimized()
    }

    fn center_window(&mut self) {
        // NO-OP
    }

    fn is_fullscreen(&self) -> runtime::sdl2::video::FullscreenType {
        self.underlying_window.borrow().is_fullscreen()
    }

    fn set_is_fullscreen(&mut self, fullscreen: runtime::sdl2::video::FullscreenType) {
        // Not sure if this is what we actually want.
        self.underlying_window
            .borrow_mut()
            .set_is_fullscreen(fullscreen);
    }

    fn is_resizable(&self) -> bool {
        self.resizable
    }

    fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
    }

    fn get_screen_size_in_px(&self) -> (u32, u32) {
        self.underlying_window.borrow().get_screen_size_in_px()
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    unsafe fn configure_viewport(&self, gl: &glow::Context, margins: SurfaceMargins) {
        // The drawing surface is centered horizontally and vertically after taking margins into account.
        // The aspect ratio of the drawing surface is preserved, and it is scaled to fit inside the window.
        let surface_size = self
            .underlying_window
            .borrow()
            .get_drawable_size_in_hardware_px();
        let available_size = (
            surface_size.0 as f32 - margins.left - margins.right,
            surface_size.1 as f32 - margins.top - margins.bottom,
        );

        let (final_width, final_height, offset_x, offset_y) = {
            let desired_ratio = (self.target_size.0 as f32) / (self.target_size.1 as f32);
            let available_ratio = available_size.0 / available_size.1;

            if available_ratio > desired_ratio {
                // Available space is wider than desired, bound by height
                let h = available_size.1;
                let w = h * desired_ratio;
                (w, h, (available_size.0 - w) / 2.0, 0.0)
            } else {
                // Available space is taller than desired, bound by width
                let w = available_size.0;
                let h = w / desired_ratio;
                (w, h, 0.0, (available_size.1 - h) / 2.0)
            }
        };

        unsafe {
            gl.viewport(
                (margins.left + offset_x).round() as i32,
                (margins.bottom + offset_y).round() as i32,
                final_width.round() as i32,
                final_height.round() as i32,
            );
        }
    }

    fn convert_sdl_to_opengl_coordinates(
        &self,
        x: f32,
        y: f32,
        margins: &SurfaceMargins,
    ) -> (f32, f32) {
        // Because the GameDrawingSurface is centered inside the underlying window, we need to adjust the coordinates accordingly.
        // This is a similar computation to configure_viewport
        let density = self.density_ratio();
        let surface_size = self
            .underlying_window
            .borrow()
            .get_drawable_size_in_hardware_px();

        let x_framebuffer = x * density.0;
        let y_framebuffer = y * density.1;

        let available_size = (
            surface_size.0 as f32 - margins.left - margins.right,
            surface_size.1 as f32 - margins.top - margins.bottom,
        );

        let (final_width, final_height, offset_x, offset_y) = {
            let desired_ratio = (self.target_size.0 as f32) / (self.target_size.1 as f32);
            let available_ratio = available_size.0 / available_size.1;

            if available_ratio > desired_ratio {
                // Available space is wider than desired, bound by height
                let h = available_size.1;
                let w = h * desired_ratio;
                (w, h, (available_size.0 - w) / 2.0, 0.0)
            } else {
                // Available space is taller than desired, bound by width
                let w = available_size.0;
                let h = w / desired_ratio;
                (w, h, 0.0, (available_size.1 - h) / 2.0)
            }
        };

        let x0 = (margins.left + offset_x).round();
        let y0 = (margins.top + offset_y).round();
        let w = final_width;
        let h = final_height;

        (
            2.0 * (x_framebuffer - x0) / w - 1.0,
            -2.0 * (y_framebuffer - y0) / h + 1.0,
        )
    }

    fn get_size_in_vectarine_px(&self) -> (f32, f32) {
        let parent_size = self
            .underlying_window
            .borrow()
            .get_drawable_size_in_hardware_px();
        let size_ratio = (
            self.target_size.0 as f32 / parent_size.0 as f32,
            self.target_size.1 as f32 / parent_size.1 as f32,
        );
        let pixel_parent_size = self.underlying_window.borrow().get_size_in_vectarine_px();
        (
            pixel_parent_size.0 * size_ratio.0,
            pixel_parent_size.1 * size_ratio.1,
        )
    }
}
