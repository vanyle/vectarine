use std::sync::Arc;

use glow::HasContext;

pub fn draw_with_mask<F, G>(gl: &Arc<glow::Context>, draw_mask: F, draw_content: G)
where
    F: FnOnce(),
    G: FnOnce(),
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
    draw_mask();

    unsafe {
        gl.stencil_mask(0x00); // Disable writing to stencil buffer
        gl.color_mask(true, true, true, true);
        gl.stencil_func(glow::EQUAL, 1, 0xFF);
        gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
    }
    draw_content();

    unsafe {
        gl.disable(glow::STENCIL_TEST);
    }
}
