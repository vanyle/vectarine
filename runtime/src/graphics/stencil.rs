use crate::graphics::{gfx::Gfx, pipeline::StencilMode};

/// Runs `draw_content`, whose drawings only appear where `draw_mask` drew.
/// Both functions need to send their drawings (flush their batch) before they return.
pub fn draw_with_mask<F, G, A, B>(gfx: &Gfx, draw_mask: F, draw_content: G) -> (A, B)
where
    F: FnOnce() -> A,
    G: FnOnce() -> B,
{
    gfx.clear_stencil();
    gfx.set_stencil_mode(StencilMode::WriteMask);
    let a = draw_mask();

    gfx.set_stencil_mode(StencilMode::TestMask);
    let b = draw_content();

    gfx.set_stencil_mode(StencilMode::Disabled);
    (a, b)
}
