use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::headless::GameHeadlessRunner;

/// Takes a screenshot of the game at the given project path, after running it for a few frames to let it initialize.
pub fn take_screenshot(
    project_path: &Path,
    output_path: Option<&Path>,
    initialization_frames: usize,
) -> vectarine_plugin_sdk::anyhow::Result<PathBuf> {
    let canonicalized_project_path = project_path.canonicalize()?;

    let game_runner = GameHeadlessRunner::new(&canonicalized_project_path);
    let mut game_runner = game_runner?;

    let frame_60th = Duration::from_secs_f32(1.0 / 60.0);
    let no_events = Vec::new();

    // We run a few frames to let the game initialize.
    for _ in 0..initialization_frames {
        game_runner.step(frame_60th, &no_events);
    }

    let output_path: Cow<Path> = match output_path {
        Some(path) => Cow::Borrowed(path),
        None => Cow::Owned(
            canonicalized_project_path
                .parent()
                .expect("Failed to get parent directory of project path")
                .join("screenshot.png"),
        ),
    };

    take_png_screenshot_from_runner(&mut game_runner, &output_path)
}

pub fn take_png_screenshot_from_runner(
    game_runner: &mut GameHeadlessRunner,
    output_path: &Path,
) -> vectarine_plugin_sdk::anyhow::Result<PathBuf> {
    let (screenshot_data, width, height) = game_runner.screenshot()?;

    let color_type = runtime::image::ColorType::Rgba8;
    let format = runtime::image::ImageFormat::Png;

    let flipped_data = screenshot_data
        .chunks_exact((width * 4) as usize)
        .rev()
        .flat_map(|row| row.to_vec())
        .collect::<Vec<u8>>();

    runtime::image::save_buffer_with_format(
        output_path,
        &flipped_data,
        width,
        height,
        color_type,
        format,
    )?;

    Ok(output_path.to_path_buf())
}
