/// This file contains functions to get paths of assets that are supposed to be shipped with the editor.
/// It contains additional logic to be able to work seamlessly in dev mode (where the executable is inside the rust target folder).
use std::{
    env,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

/// An editor asset is a static file that is supposed to be provided with the editor installation and is usually located next to the editor executable.
fn get_editor_asset_path(asset_filename: &str) -> PathBuf {
    let executable_path = std::env::current_exe().unwrap_or_default();
    let executable_parent = executable_path.parent().unwrap_or(Path::new("."));
    let asset_path = executable_parent.join(asset_filename);
    if asset_path.exists() {
        return asset_path;
    }
    // Fallback to current working directory
    let cwd = std::env::current_dir().unwrap_or_default();
    cwd.join(asset_filename)
}

pub fn get_gallery_path() -> PathBuf {
    get_editor_asset_path("gallery")
}

pub fn get_luau_api_path() -> PathBuf {
    get_editor_asset_path("luau-api")
}

fn look_for_file_next_to_exe(locations: &[&str], file_name: Option<&str>) -> Option<PathBuf> {
    let exec_path = env::current_exe().ok()?;
    let exec_dir = exec_path.parent()?;
    for loc in locations {
        let candidate = exec_dir.join(loc);
        let candidate = if let Some(file_name) = file_name {
            candidate.join(file_name)
        } else {
            candidate
        };
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn get_runtime_file_paths_for_web()
-> Option<(std::path::PathBuf, std::path::PathBuf, std::path::PathBuf)> {
    let locations_to_check = [
        ".",
        "../wasm32-unknown-emscripten/release",
        "../wasm32-unknown-emscripten/debug",
        "../..",
    ];
    let runtime_js_path = look_for_file_next_to_exe(&locations_to_check, Some("runtime.js"));
    let runtime_wasm_path = look_for_file_next_to_exe(&locations_to_check, Some("runtime.wasm"));
    let index_html_path = look_for_file_next_to_exe(&locations_to_check, Some("index.html"));

    if let Some(runtime) = runtime_js_path
        && let Some(wasm) = runtime_wasm_path
        && let Some(index) = index_html_path
    {
        Some((runtime, wasm, index))
    } else {
        None
    }
}

pub fn get_runtime_file_for_windows() -> Option<PathBuf> {
    look_for_file_next_to_exe(
        &[
            "./runtime.exe",
            "../x86_64-pc-windows-msvc/release/runtime.exe",
            "../x86_64-pc-windows-msvc/debug/runtime.exe",
            "../release/runtime.exe",
        ],
        None,
    )
}

pub fn get_runtime_file_for_linux() -> Option<PathBuf> {
    look_for_file_next_to_exe(
        &[
            "./runtime-linux",
            "../x86_64-unknown-linux-gnu/release/runtime",
            "../x86_64-unknown-linux-gnu/debug/runtime",
            // No runtime because it is ambiguous if we are on mac or linux
            // "../release/runtime",
        ],
        None,
    )
}

pub fn get_runtime_file_for_macos() -> Option<PathBuf> {
    look_for_file_next_to_exe(
        &[
            "./runtime-macos",
            "../x86_64-apple-darwin/release/runtime",
            "../x86_64-apple-darwin/debug/runtime",
        ],
        None,
    )
}

const EDITOR_CONFIG_FILE: &str = "vectarine-config.toml";

fn get_base_dir() -> ProjectDirs {
    directories::ProjectDirs::from("com", "vanyle", "vectarine")
        .expect("Your operating system is not supported by vectarine.")
}

pub fn get_editor_config_path() -> PathBuf {
    let base_dirs = get_base_dir();
    base_dirs.config_dir().join(EDITOR_CONFIG_FILE)
}

pub fn get_editor_plugins_path() -> PathBuf {
    let base_dirs = get_base_dir();
    base_dirs.data_dir().join("plugins")
}

pub static PLUGIN_FILE_EXTENSION: &str = ".vectaplugin";

pub fn does_path_end_with(path: &Path, suffix: &str) -> bool {
    path.to_string_lossy().ends_with(suffix)
}
