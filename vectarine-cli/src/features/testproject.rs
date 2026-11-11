use runtime::image::{
    Delay, Frame,
    codecs::gif::{GifEncoder, Repeat},
};
use runtime::{
    anyhow::{self, Result, anyhow},
    image::RgbaImage,
    sdl2::{self, event::Event},
    toml,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

use crate::{
    features::testproject::testfileparsing::{TestFile, TestStep},
    headless::{FrameResult, GameHeadlessRunner},
};

mod testfileparsing {
    use runtime::serde::{Deserialize, Serialize};
    use std::path::PathBuf;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(crate = "runtime::serde")]
    pub(crate) struct TestFile {
        pub project: Project,
        pub step: Vec<TestStep>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(crate = "runtime::serde")]
    pub(crate) struct Project {
        pub path: PathBuf,
        pub description: Option<String>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(crate = "runtime::serde")]
    pub(crate) enum TestStep {
        #[serde(rename = "wait_for_frames")]
        Idle(u32),
        #[serde(rename = "compare_screenshot_to")]
        CompareScreenshot(String),
        #[serde(rename = "compare_logs_to")]
        CompareLogs(String),
        #[serde(rename = "expect_no_errors")]
        ExpectNoErrors,
        #[serde(rename = "clear_logs")]
        ClearLogs,
        #[serde(rename = "press_keys")]
        ActionKeyPress(Vec<String>),
        #[serde(rename = "release_keys")]
        ActionKeyRelease(Vec<String>),
        #[serde(rename = "tap_keys")]
        ActionKeyTap(Vec<String>),
        #[serde(rename = "press_mouse_at")]
        ActionMousePress(i32, i32),
        #[serde(rename = "release_mouse_at")]
        ActionMouseRelease(i32, i32),
        #[serde(rename = "run_lua_code")]
        RunLuaCode(String),
        #[serde(rename = "record_gif")]
        RecordGif { frames: u32, output: String },
    }
}

/// Tries to make a relative path absolute using the second path provided.
fn make_path_absolute(relative_file_path: &Path, anchor_file_path: &Path) -> PathBuf {
    if let Some(parent_dir) = anchor_file_path.parent() {
        let joined = parent_dir.join(relative_file_path);
        if let Ok(path) = joined.canonicalize() {
            return path;
        }
        return joined;
    }

    if let Ok(path) = relative_file_path.canonicalize() {
        return path;
    }

    relative_file_path.to_path_buf()
}

fn push_key_events(
    event_buffer: &mut Vec<Event>,
    key_names: &[String],
    down: bool,
    window_id: u32,
) {
    for key_name in key_names {
        let scancode = sdl2::keyboard::Scancode::from_name(key_name);
        let keycode = sdl2::keyboard::Keycode::from_name(key_name);
        if keycode.is_none() {
            println!(
                "Warning: Key name '{}' not recognized. Check https://wiki.libsdl.org/SDL2/SDL_KeyCode for valid key names.",
                key_name
            );
        }
        let event = if down {
            Event::KeyDown {
                timestamp: 0,
                window_id,
                keycode,
                scancode,
                keymod: sdl2::keyboard::Mod::empty(),
                repeat: false,
            }
        } else {
            Event::KeyUp {
                timestamp: 0,
                window_id,
                keycode,
                scancode,
                keymod: sdl2::keyboard::Mod::empty(),
                repeat: false,
            }
        };
        event_buffer.push(event);
    }
}

enum Encoder {
    Gif(GifEncoder<BufWriter<File>>),
    Mp4(Child),
}

struct Recording {
    output: PathBuf,
    frame_count: usize,
    frames_recorded: usize,
    encoder: Option<Encoder>,
}

impl Recording {
    fn new(output: PathBuf, frame_count: usize) -> Result<Self> {
        if !matches!(
            output.extension().and_then(|e| e.to_str()),
            Some("gif" | "mp4")
        ) {
            return Err(anyhow!(
                "Unsupported recording format for {}: use .gif or .mp4",
                output.display()
            ));
        }
        Ok(Self {
            output,
            frame_count,
            frames_recorded: 0,
            encoder: None,
        })
    }

    fn start_encoder(&self, width: u32, height: u32) -> Result<Encoder> {
        if self.output.extension().and_then(|e| e.to_str()) == Some("gif") {
            let file = BufWriter::new(File::create(&self.output)?);
            let mut gif = GifEncoder::new_with_speed(file, 10);
            gif.set_repeat(Repeat::Infinite)?;
            return Ok(Encoder::Gif(gif));
        }
        let ffmpeg = Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba", "-s"])
            .arg(format!("{width}x{height}"))
            .args(["-r", "60", "-i", "-", "-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2"])
            .args(["-c:v", "libx264", "-crf", "12", "-pix_fmt", "yuv420p"])
            .args(["-movflags", "+faststart"])
            .arg(&self.output)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| {
                anyhow!("Recording an mp4 requires ffmpeg in your PATH ({e}). Record a .gif instead, or install ffmpeg.")
            })?;
        Ok(Encoder::Mp4(ffmpeg))
    }

    fn add_frame(&mut self, frame: &RgbaImage) -> Result<()> {
        if self.encoder.is_none() {
            self.encoder = Some(self.start_encoder(frame.width(), frame.height())?);
        }
        match &mut self.encoder {
            // GIF delays are in centiseconds and browsers slow down delays under 2, so we keep 30 fps.
            Some(Encoder::Gif(gif)) if self.frames_recorded.is_multiple_of(2) => {
                let centiseconds_at = |frame_index: usize| (frame_index as u32 * 100 + 30) / 60;
                let delay = centiseconds_at(self.frames_recorded + 2)
                    - centiseconds_at(self.frames_recorded);
                gif.encode_frame(Frame::from_parts(
                    frame.clone(),
                    0,
                    0,
                    Delay::from_numer_denom_ms(delay * 10, 1),
                ))?;
            }
            Some(Encoder::Mp4(ffmpeg)) => {
                ffmpeg
                    .stdin
                    .as_mut()
                    .ok_or_else(|| anyhow!("Failed to send the frames to ffmpeg"))?
                    .write_all(frame.as_raw())?;
            }
            _ => {}
        }
        self.frames_recorded += 1;
        Ok(())
    }

    fn finish(self) -> Result<()> {
        match self.encoder {
            None => {
                println!(
                    "Warning: no frames were recorded for {}",
                    self.output.display()
                );
                return Ok(());
            }
            Some(Encoder::Gif(gif)) => drop(gif),
            Some(Encoder::Mp4(mut ffmpeg)) => {
                drop(ffmpeg.stdin.take());
                if !ffmpeg.wait()?.success() {
                    return Err(anyhow!("ffmpeg failed to write {}", self.output.display()));
                }
            }
        }
        println!(
            "Recorded {} frames to {}",
            self.frames_recorded,
            self.output.display()
        );
        Ok(())
    }
}

fn capture_screenshot(game_runner: &GameHeadlessRunner) -> Result<RgbaImage> {
    let (data, width, height) = game_runner.screenshot()?;
    RgbaImage::from_raw(width, height, data)
        .ok_or_else(|| anyhow!("Failed to create image buffer from screenshot data"))
}

fn step_frame(
    game_runner: &mut GameHeadlessRunner,
    event_buffer: &mut Vec<Event>,
    recordings: &mut Vec<Recording>,
) -> Result<FrameResult> {
    let result = game_runner.step(Duration::from_secs_f32(1.0 / 60.0), event_buffer);
    event_buffer.clear();
    if !recordings.is_empty() {
        let frame = capture_screenshot(game_runner)?;
        for recording in recordings.iter_mut() {
            recording.add_frame(&frame)?;
        }
    }
    let (finished, remaining): (Vec<Recording>, Vec<Recording>) = std::mem::take(recordings)
        .into_iter()
        .partition(|recording| recording.frames_recorded >= recording.frame_count);
    *recordings = remaining;
    for recording in finished {
        recording.finish()?;
    }
    Ok(result)
}

pub fn test_project(test_file: &Path, overwrite: bool, acceptable_pixel_diff: u32) -> Result<()> {
    if test_file.is_dir() {
        // Recursively find all *vecta-test.toml files in the directory and run them.
        let entries = std::fs::read_dir(test_file)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                test_project(&path, overwrite, acceptable_pixel_diff)?;
            } else if path.is_file() && path.to_string_lossy().ends_with("vecta-test.toml") {
                run_test_file(&path, overwrite, acceptable_pixel_diff)?;
            }
        }
        Ok(())
    } else if test_file.is_file() {
        run_test_file(test_file, overwrite, acceptable_pixel_diff)
    } else {
        Err(anyhow::anyhow!(
            "The test file {:?} does not exist",
            test_file
        ))
    }
}

pub fn run_test_file(test_file: &Path, overwrite: bool, acceptable_pixel_diff: u32) -> Result<()> {
    let test_file_content = std::fs::read(test_file).expect("Failed to read test file");

    let test_manifest = toml::from_slice::<TestFile>(&test_file_content)?;

    let canonicalized_project_path = make_path_absolute(&test_manifest.project.path, test_file);
    let canon_string = canonicalized_project_path.to_string_lossy().into_owned();

    println!("Testing: {} ...", canon_string);
    if let Some(description) = &test_manifest.project.description {
        println!("➡️ {}", description);
    }

    let game_runner = GameHeadlessRunner::new(&canonicalized_project_path);
    let mut game_runner = game_runner?;
    let mut event_buffer = vec![];
    let mut logs = vec![];
    let mut recordings = vec![];

    for step in test_manifest.step {
        match step {
            TestStep::Idle(count) => {
                for _ in 0..count {
                    let result = step_frame(&mut game_runner, &mut event_buffer, &mut recordings)?;
                    logs.extend(result.logs);
                }
            }
            TestStep::CompareScreenshot(path) => {
                let screenshot_output_path = make_path_absolute(Path::new(&path), test_file);
                let actual_bytes = capture_screenshot(&game_runner)?;
                let (width, height) = actual_bytes.dimensions();

                if !screenshot_output_path.exists() || overwrite {
                    actual_bytes.save_with_format(
                        screenshot_output_path,
                        runtime::image::ImageFormat::Png,
                    )?;
                } else {
                    let expected_image = runtime::image::open(&screenshot_output_path)?;
                    let expected_width = expected_image.width();
                    let expected_height = expected_image.height();
                    if width != expected_width || height != expected_height {
                        return Err(anyhow!(
                            "There was a difference between the size of the screenshot taken and the saved one at {}: expected {}x{}, got {}x{}",
                            screenshot_output_path.display(),
                            expected_width,
                            expected_height,
                            width,
                            height
                        ));
                    }

                    // We blur slightly due to differences in multisampling between platforms. This is a tradeoff between accuracy and robustness.
                    let expected_bytes = expected_image.to_rgba8();

                    let total_pixels = width * height;
                    let mut differing_pixels = 0;
                    for x in 0..width {
                        for y in 0..height {
                            let expected_pixel = expected_bytes.get_pixel(x, y);
                            let actual_pixel = actual_bytes.get_pixel(x, y);

                            if expected_pixel != actual_pixel {
                                differing_pixels += 1;
                            }
                        }
                    }
                    let pixel_difference_percentage =
                        (differing_pixels as f32 / total_pixels as f32) * 100.0;
                    if pixel_difference_percentage > acceptable_pixel_diff as f32 {
                        return Err(anyhow!(
                            "There was a difference between the screenshot taken and the saved one at {}: {}% of pixels differ, which is above the acceptable threshold of {}%",
                            screenshot_output_path.display(),
                            pixel_difference_percentage,
                            acceptable_pixel_diff
                        ));
                    } else {
                        println!(
                            "Screenshot comparison passed: {}% of pixels differ, which is within the acceptable threshold of {}%",
                            pixel_difference_percentage, acceptable_pixel_diff
                        );
                    }
                }
            }
            TestStep::RunLuaCode(code) => {
                game_runner.run_lua_code(&code)?;
                logs.extend(game_runner.take_logs().logs);
            }
            TestStep::CompareLogs(path) => {
                let log_strings: Vec<String> = logs
                    .iter()
                    .map(|log| match log {
                        runtime::console::ConsoleMessage::Info(msg) => format!("Info: {}", msg),
                        runtime::console::ConsoleMessage::Warning(msg) => {
                            format!("Warning: {}", msg)
                        }
                        runtime::console::ConsoleMessage::Error(msg) => format!("Error: {}", msg),
                        runtime::console::ConsoleMessage::LuaError(msg) => {
                            format!("Lua Error: {}", msg)
                        }
                        runtime::console::ConsoleMessage::Reload => "--- Reload ---".to_string(),
                    })
                    .collect();

                let log_path = make_path_absolute(Path::new(&path), test_file);

                if !log_path.exists() || overwrite {
                    // Save the logs to the file if it doesn't exist as there is nothing to compare to.
                    std::fs::write(&log_path, log_strings.join("\n"))
                        .map_err(|_| anyhow!("Failed to write logs to file."))?;
                } else {
                    // Compare the logs line-by-line to the expected logs in the file.
                    let expected_logs = std::fs::read_to_string(&log_path)
                        .map_err(|_| anyhow!("Failed to read logs from file."))?;
                    let expected_logs: Vec<&str> = expected_logs.lines().collect();
                    if expected_logs.len() != log_strings.len() {
                        println!(
                            "Log length mismatch: expected {} lines, got {} lines",
                            expected_logs.len(),
                            log_strings.len()
                        );
                        return Err(anyhow!("Log comparison failed."));
                    }
                    for (i, (expected, actual)) in
                        expected_logs.iter().zip(log_strings.iter()).enumerate()
                    {
                        if expected != actual {
                            println!(
                                "Log mismatch at line {}: expected '{}', got '{}'",
                                i + 1,
                                expected,
                                actual
                            );
                            return Err(anyhow!("Log comparison failed."));
                        }
                    }
                }
            }
            TestStep::ClearLogs => {
                logs.clear();
            }
            TestStep::ExpectNoErrors => {
                let mut errors_found = false;
                for log in &logs {
                    match log {
                        runtime::console::ConsoleMessage::Error(msg) => {
                            errors_found = true;
                            println!("Error: {}", msg);
                        }
                        runtime::console::ConsoleMessage::LuaError(msg) => {
                            errors_found = true;
                            println!("Lua Error: {}", msg);
                        }
                        _ => (),
                    }
                }
                if errors_found {
                    println!("Errors were found during test execution, see above.");
                    return Err(anyhow!("Test failed due to errors."));
                }
            }
            TestStep::ActionKeyPress(key_names) => {
                push_key_events(&mut event_buffer, &key_names, true, game_runner.window_id());
            }
            TestStep::ActionKeyRelease(key_names) => {
                push_key_events(
                    &mut event_buffer,
                    &key_names,
                    false,
                    game_runner.window_id(),
                );
            }
            TestStep::ActionKeyTap(key_names) => {
                push_key_events(&mut event_buffer, &key_names, true, game_runner.window_id());
                let result = step_frame(&mut game_runner, &mut event_buffer, &mut recordings)?;
                logs.extend(result.logs);
                // The keys are released on the next frame, like a release_keys step.
                push_key_events(
                    &mut event_buffer,
                    &key_names,
                    false,
                    game_runner.window_id(),
                );
            }
            TestStep::RecordGif { frames, output } => {
                let output = make_path_absolute(Path::new(&output), test_file);
                recordings.push(Recording::new(output, frames as usize)?);
            }
            TestStep::ActionMousePress(x, y) => {
                event_buffer.push(Event::MouseButtonDown {
                    timestamp: 0,
                    window_id: game_runner.window_id(),
                    mouse_btn: sdl2::mouse::MouseButton::Left,
                    which: 0,
                    clicks: 1,
                    x,
                    y,
                });
            }
            TestStep::ActionMouseRelease(x, y) => {
                event_buffer.push(Event::MouseButtonUp {
                    timestamp: 0,
                    window_id: game_runner.window_id(),
                    mouse_btn: sdl2::mouse::MouseButton::Left,
                    which: 0,
                    clicks: 1,
                    x,
                    y,
                });
            }
        }
    }

    for recording in recordings {
        println!(
            "Warning: the test ended after {} of the {} frames to record in {}",
            recording.frames_recorded,
            recording.frame_count,
            recording.output.display()
        );
        recording.finish()?;
    }

    Ok(())
}
