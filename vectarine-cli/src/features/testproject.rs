use runtime::{
    anyhow::{Result, anyhow},
    sdl2::{self, event::Event},
    toml,
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    features::{
        screenshot::take_png_screenshot_from_runner,
        testproject::testfileparsing::{TestFile, TestStep},
    },
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
        #[serde(rename = "save_screenshot_to")]
        Screenshot(String),
        #[serde(rename = "save_logs_to")]
        SaveLogs(String),
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

pub fn test_project(test_file: &Path) -> Result<()> {
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
            TestStep::Screenshot(path) => {
                let path = make_path_absolute(Path::new(&path), test_file);
                take_png_screenshot_from_runner(&mut game_runner, &path)?;
            }
            TestStep::RunLuaCode(code) => {
                game_runner.run_lua_code(&code)?;
            }
            TestStep::SaveLogs(path) => {
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
                std::fs::write(&log_path, log_strings.join("\n"))
                    .map_err(|_| anyhow!("Failed to write logs to file."))?;
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
