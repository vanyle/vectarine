use runtime::{
    anyhow::{self, Result, anyhow},
    sdl2::{self, event::Event},
    toml,
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    features::testproject::testfileparsing::{TestFile, TestStep},
    headless::GameHeadlessRunner,
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
        #[serde(rename = "press_mouse_at")]
        ActionMousePress(i32, i32),
        #[serde(rename = "release_mouse_at")]
        ActionMouseRelease(i32, i32),
        #[serde(rename = "run_lua_code")]
        RunLuaCode(String),
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

pub fn test_project(test_file: &Path, overwrite: bool) -> Result<()> {
    if test_file.is_dir() {
        // Recursively find all *vecta-test.toml files in the directory and run them.
        let entries = std::fs::read_dir(test_file)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                test_project(&path, overwrite)?;
            } else if path.is_file() && path.to_string_lossy().ends_with("vecta-test.toml") {
                run_test_file(&path, overwrite)?;
            }
        }
        Ok(())
    } else if test_file.is_file() {
        run_test_file(test_file, overwrite)
    } else {
        Err(anyhow::anyhow!(
            "The test file {:?} does not exist",
            test_file
        ))
    }
}

pub fn run_test_file(test_file: &Path, overwrite: bool) -> Result<()> {
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

    for step in test_manifest.step {
        match step {
            TestStep::Idle(count) => {
                for _ in 0..count {
                    let result =
                        game_runner.step(Duration::from_secs_f32(1.0 / 60.0), &event_buffer);
                    logs.extend(result.logs);
                    event_buffer.clear();
                }
            }
            TestStep::CompareScreenshot(path) => {
                let screenshot_output_path = make_path_absolute(Path::new(&path), test_file);
                let (screenshot_data, width, height) = game_runner.screenshot()?;

                if !screenshot_output_path.exists() || overwrite {
                    let flipped_data = screenshot_data
                        .chunks_exact((width * 4) as usize)
                        .rev()
                        .flat_map(|row| row.to_vec())
                        .collect::<Vec<u8>>();

                    runtime::image::save_buffer_with_format(
                        screenshot_output_path,
                        &flipped_data,
                        width,
                        height,
                        runtime::image::ColorType::Rgba8,
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
                    let bytes = expected_image.to_rgba8();
                    for x in 0..width {
                        for y in 0..height {
                            let expected_pixel = bytes.get_pixel(x, y);
                            let flipped_index = ((height - 1 - y) * width + x) as usize;
                            let actual_pixel =
                                &screenshot_data[flipped_index * 4..(flipped_index + 1) * 4];
                            if expected_pixel.0 != actual_pixel {
                                return Err(anyhow!(
                                    "There was a difference in the pixel data of the screenshot taken and the saved one at {}: expected {:?}, got {:?} at position ({}, {})",
                                    screenshot_output_path.display(),
                                    expected_pixel.0,
                                    actual_pixel,
                                    x,
                                    y
                                ));
                            }
                        }
                    }
                }
            }
            TestStep::RunLuaCode(code) => {
                game_runner.run_lua_code(&code)?;
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
                    print!(
                        "The logs seem to match the expected logs in {}.",
                        log_path.display()
                    );
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
                for key_name in key_names {
                    let scancode = sdl2::keyboard::Scancode::from_name(&key_name);
                    let keycode = sdl2::keyboard::Keycode::from_name(&key_name);
                    event_buffer.push(Event::KeyDown {
                        timestamp: 0,
                        window_id: game_runner.window_id(),
                        keycode,
                        scancode,
                        keymod: sdl2::keyboard::Mod::empty(),
                        repeat: false,
                    });
                }
            }
            TestStep::ActionKeyRelease(key_names) => {
                for key_name in key_names {
                    let keycode = sdl2::keyboard::Keycode::from_name(&key_name);
                    let scancode = keycode.and_then(sdl2::keyboard::Scancode::from_keycode);
                    if keycode.is_none() {
                        println!(
                            "Warning: Key name '{}' not recognized. Check https://wiki.libsdl.org/SDL2/SDL_KeyCode for valid key names.",
                            key_name
                        );
                    }
                    event_buffer.push(Event::KeyUp {
                        timestamp: 0,
                        window_id: game_runner.window_id(),
                        keycode,
                        scancode,
                        keymod: sdl2::keyboard::Mod::empty(),
                        repeat: false,
                    });
                }
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

    Ok(())
}
