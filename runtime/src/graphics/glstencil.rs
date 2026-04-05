use std::sync::Arc;

use vectarine_plugin_sdk::glow;
use vectarine_plugin_sdk::glow::HasContext;

pub fn draw_with_mask<F, G, A, B>(gl: &Arc<glow::Context>, draw_mask: F, draw_content: G) -> (A, B)
where
    F: FnOnce() -> A,
    G: FnOnce() -> B,
{
    unsafe {
        gl.enable(glow::STENCIL_TEST);
        gl.stencil_mask(0xFF); // Enable writing to the stencil buffer
        gl.clear_stencil(0); // Explicitly clear to 0
        gl.clear(glow::STENCIL_BUFFER_BIT);
        gl.color_mask(false, false, false, false); // Don't draw to the screen
        gl.stencil_func(glow::ALWAYS, 1, 0xFF);
        gl.stencil_op(glow::REPLACE, glow::REPLACE, glow::REPLACE);
    }
    let a = draw_mask();

    unsafe {
        gl.stencil_mask(0x00); // Disable writing to stencil buffer
        gl.color_mask(true, true, true, true);
        gl.stencil_func(glow::EQUAL, 1, 0xFF);
        gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
    }
    let b = draw_content();

    unsafe {
        gl.disable(glow::STENCIL_TEST);
    }
    (a, b)
}
