use std::path::Path;

use vectarine_plugin_sdk::glow;

pub fn take_screenshot(project_path: &Path, output_path: Option<&Path>) {
    // We'll need to have a headless opengl context,
    // and use traits for sound and input instead of the concrete sdl2 implems.
    // Probably at some point we'll want to record video based on prerecorded inputs, so
    // this will be needed anyway.
}

pub fn get_image_data_from_framebuffer(framebuffer: glow::Framebuffer) -> Vec<u8> {
    vec![]
}
