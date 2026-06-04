use std::path::Path;

pub fn take_screenshot(project_path: &Path, output_path: Option<&Path>) {
    // We'll need to have a headless opengl context,
    // and use traits for sound and input instead of the concrete sdl2 implems.
    // Probably at some point we'll want to record video based on prerecorded inputs, so
    // this will be needed anyway.
}
