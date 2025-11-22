use lazy_static::lazy_static;
use regex::Regex;
use runtime::mlua;
use runtime::projectinfo::ProjectInfo;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;
use std::thread;
use std::{cell::RefCell, env, fs, path::PathBuf};

use egui::{Color32, RichText, TextBuffer, Widget};
use zip::write::SimpleFileOptions;

use crate::editorinterface::EditorState;

#[derive(PartialEq, Clone, Copy)]
enum ExportPlatform {
    Windows,
    Linux,
    MacOS,
    Web,
}

pub fn draw_editor_export(editor: &mut EditorState, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow().is_export_window_shown;

    egui::Window::new("Export Project")
        .default_width(600.0)
        .default_height(400.0)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut is_shown)
        .show(ctx, |ui| {
            draw_editor_export_window(ui, editor);
        });
    editor.config.borrow_mut().is_export_window_shown = is_shown;
}

fn ui_title(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).heading());
}

fn draw_editor_export_window(ui: &mut egui::Ui, editor: &mut EditorState) {
    let mut project = editor.project.borrow_mut();
    let project = project.as_mut();
    let Some(project) = project else {
        ui.label("No project is currently loaded.");
        return;
    };

    let project_file_path = &mut project.project_path;
    let project_folder = project_file_path
        .parent()
        .expect("Failed to get project folder");

    thread_local! {
        static OBFUSCATE_GAME_DATA: RefCell<bool> = const { RefCell::new(true) };
        static TARGET_PLATFORM: RefCell<ExportPlatform> = const { RefCell::new(ExportPlatform::Web) };
    }

    ui_title(ui, "Optimization");

    OBFUSCATE_GAME_DATA.with_borrow_mut(|obfuscate_game_data| {
        const OBFUSCATION_INFO: &str = "
Obfuscation compresses your game and makes it run faster. \
The content of the distributed version becomes unreadable and uneditable by third-parties.
Read the manual section about obfuscation for more details.
        ";
        ui.checkbox(obfuscate_game_data, "Obfuscate game data")
            .on_hover_text(OBFUSCATION_INFO);
    });

    // -----------------
    ui.add_space(8.0);
    ui_title(ui, "Export platform");
    ui.horizontal_wrapped(|ui| {
        TARGET_PLATFORM.with_borrow_mut(|target_platform| {
            ui.selectable_value(target_platform, ExportPlatform::Windows, "Windows");
            ui.selectable_value(target_platform, ExportPlatform::Linux, "Linux");
            ui.selectable_value(target_platform, ExportPlatform::MacOS, "macOS");
            ui.selectable_value(target_platform, ExportPlatform::Web, "Web");
        });
    });

    // -----------------
    ui.add_space(8.0);

    ui_title(ui, "Export folder");
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Open export folder")
            .on_hover_text("Open the folder where the exported game will be saved.")
            .clicked()
        {
            let _ = open::that(project_folder);
        }
        ui.label(
            RichText::new(project_folder.display().to_string())
                .monospace()
                .color(Color32::WHITE)
                .background_color(Color32::from_gray(0x22)),
        );
    });

    // -----------------
    ui.add_space(8.0);

    let export_button = egui::Button::new(RichText::new("Export").size(20.0));

    lazy_static! {
        static ref EXPORT_LOG_BUFFER: Mutex<String> = Mutex::new(String::new());
    }

    if export_button.ui(ui).clicked() {
        {
            let mut log_buffer = EXPORT_LOG_BUFFER.lock().expect("Failed to lock log buffer");
            log_buffer.clear();
        }
        let project_path = project.project_path.clone();
        let project_info = project.project_info.clone();
        let obfuscate_data = OBFUSCATE_GAME_DATA.with_borrow(|b| *b);
        let target_platform = TARGET_PLATFORM.with_borrow(|p| *p);

        thread::spawn(move || {
            let result = export_project(
                &project_path,
                &project_info,
                obfuscate_data,
                target_platform,
            );
            if let Err(err_msg) = result {
                let mut log_buffer = EXPORT_LOG_BUFFER.lock().expect("Failed to lock log buffer");
                *log_buffer = format!("Export failed: {}\n", err_msg);
            } else {
                let mut log_buffer = EXPORT_LOG_BUFFER.lock().expect("Failed to lock log buffer");
                *log_buffer = "Export completed successfully.\n".into();
            }
        });
    }
    {
        if let Ok(log_buffer) = EXPORT_LOG_BUFFER.try_lock()
            && !log_buffer.is_empty()
        {
            ui.add_space(8.0);
            ui.label(RichText::new(&*log_buffer).monospace());
        }
    }
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

fn get_runtime_file_paths_for_web()
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

fn add_file_to_zip_from_path(
    zip: &mut zip::ZipWriter<fs::File>,
    file_path: &Path,
    zip_path: &str,
    as_executable: bool,
    as_zstd: bool,
) -> std::io::Result<()> {
    let options = SimpleFileOptions::default();
    let options = if as_executable {
        options.unix_permissions(0o755)
    } else {
        options
    };
    let options = if as_zstd {
        options.compression_method(zip::CompressionMethod::Zstd)
    } else {
        options
    };

    zip.start_file(zip_path, options)?;
    let mut f = fs::File::open(file_path)?;
    io::copy(&mut f, zip)?;
    Ok(())
}

fn add_file_content_to_zip(
    zip: &mut zip::ZipWriter<fs::File>,
    content: &[u8],
    zip_path: &str,
    options: SimpleFileOptions,
) -> std::io::Result<()> {
    zip.start_file(zip_path, options)?;
    zip.write_all(content)?;
    Ok(())
}

fn get_export_filename(project_info: &ProjectInfo, platform: ExportPlatform) -> String {
    let project_name = &project_info.title.replace(" ", "_");
    match platform {
        ExportPlatform::Windows => format!("{}_windows.zip", project_name),
        ExportPlatform::Linux => format!("{}_linux.zip", project_name),
        ExportPlatform::MacOS => format!("{}_macos.zip", project_name),
        ExportPlatform::Web => format!("{}_web.zip", project_name),
    }
}

fn get_files_in_folder(folder_path: &Path, zip_base_path: &str) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(folder_path) else {
        return files;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if path.is_file() {
            let zip_path = format!("{}/{}", zip_base_path, file_name);
            files.push((path, zip_path));
        } else if path.is_dir() {
            let new_zip_base_path = format!("{}/{}", zip_base_path, file_name);
            let mut sub_files = get_files_in_folder(&path, &new_zip_base_path);
            files.append(&mut sub_files);
        }
    }
    files
}

fn get_project_files(project_path: &Path) -> impl Iterator<Item = (PathBuf, String)> {
    let game_data_folder = project_path
        .parent()
        .expect("Failed to get game data folder");
    let unexported_folder_names = [
        "release", "game", "output", "build", "debug", "export", "private",
    ];
    // Add game data folder
    // Adding .vecta file as executable as you can run it using a shebang.
    let mut iter = vec![(
        project_path.to_path_buf(),
        "gamedata/game.vecta".to_string(),
    )];

    let Ok(game_data_files) = fs::read_dir(game_data_folder) else {
        return iter.into_iter();
    };
    for entry in game_data_files {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(folder_name) = path.file_name() else {
            unreachable!(
                "When listing files in a directory like {}, only entries which a filename should be returned.",
                game_data_folder.display()
            );
        };
        let folder_name = folder_name.to_string_lossy();
        let folder_name = folder_name.as_str();
        if unexported_folder_names.contains(&folder_name) {
            continue;
        }
        let sub_iter = get_files_in_folder(&path, &format!("gamedata/{}", folder_name));
        iter.extend(sub_iter);
    }
    iter.into_iter()
}

fn export_project(
    project_path: &Path,
    project_info: &ProjectInfo,
    obfuscate: bool,
    platform: ExportPlatform,
) -> Result<PathBuf, String> {
    let game_data_folder = project_path
        .parent()
        .expect("Failed to get game data folder");

    let exported_filename = get_export_filename(project_info, platform);
    let output_path = game_data_folder.join(exported_filename);
    if output_path.exists() {
        let _ = fs::remove_file(&output_path);
    }

    let output_file = fs::File::create(&output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(output_file);

    match platform {
        ExportPlatform::Web => {
            let Some((runtime_js_path, runtime_wasm_path, index_html_path)) =
                get_runtime_file_paths_for_web()
            else {
                return Err(
                    "Failed to locate runtime files (runtime.js, runtime.wasm, index.html). \
            Make sure they are located next to the executable."
                        .into(),
                );
            };

            let output_path = game_data_folder.join("web_export.zip");
            if output_path.exists() {
                let _ = fs::remove_file(&output_path);
            }

            let index_html_content =
                fs::read_to_string(&index_html_path).map_err(|e| e.to_string())?;
            let re = Regex::new(r"target/[a-zA-Z0-9\-/]+/runtime.js").map_err(|e| e.to_string())?;
            let index_html_content = re.replace_all(&index_html_content, "runtime.js");
            let index_html_content =
                index_html_content.replace("Vectarine Web Build", &project_info.title);
            add_file_content_to_zip(
                &mut zip,
                index_html_content.as_bytes(),
                "index.html",
                SimpleFileOptions::default(),
            )
            .map_err(|e| e.to_string())?;

            add_file_to_zip_from_path(&mut zip, &runtime_js_path, "runtime.js", false, false)
                .map_err(|e| e.to_string())?;
            add_file_to_zip_from_path(&mut zip, &runtime_wasm_path, "runtime.wasm", false, false)
                .map_err(|e| e.to_string())?;
        }
        ExportPlatform::Windows => {
            let runtime_path = look_for_file_next_to_exe(
                &[
                    "../x86_64-pc-windows-msvc/release",
                    ".",
                    "../x86_64-pc-windows-msvc/debug",
                    "../release",
                ],
                Some("runtime.exe"),
            );
            if let Some(runtime_path) = runtime_path {
                add_file_to_zip_from_path(&mut zip, &runtime_path, "game.exe", true, false)
                    .map_err(|e| e.to_string())?;
            } else {
                return Err("Failed to locate runtime.exe".into());
            }
        }
        ExportPlatform::Linux => {
            let runtime_path = look_for_file_next_to_exe(
                &[
                    "./runtime-linux",
                    "../x86_64-unknown-linux-gnu/release/runtime",
                    "../x86_64-unknown-linux-gnu/debug/runtime",
                ],
                None,
            );
            if let Some(runtime_path) = runtime_path {
                add_file_to_zip_from_path(&mut zip, &runtime_path, "game", true, false)
                    .map_err(|e| e.to_string())?;
            } else {
                return Err("Failed to locate runtime executable".into());
            }
        }
        ExportPlatform::MacOS => {
            let runtime_path = look_for_file_next_to_exe(
                &[
                    "./runtime-macos",
                    "../x86_64-apple-darwin/release/runtime",
                    "../x86_64-apple-darwin/debug/runtime",
                ],
                None,
            );
            if let Some(runtime_path) = runtime_path {
                add_file_to_zip_from_path(&mut zip, &runtime_path, "game", true, false)
                    .map_err(|e| e.to_string())?;
            } else {
                return Err("Failed to locate runtime executable".into());
            }
        }
    }

    if !obfuscate {
        // Add game data folder
        // Adding .vecta file as executable as you can run it using a shebang.

        let game_data_files = get_project_files(project_path);
        for (file_path, zip_path) in game_data_files {
            add_file_to_zip_from_path(&mut zip, &file_path, &zip_path, false, false)
                .map_err(|e| e.to_string())?;
        }
    } else {
        // Compress game data into bundle.vecta (a zip with zstd compression)
        // then, put the bundle.vecta file into the exported zip
        let inner_zip_path = game_data_folder.join("bundle.vecta");
        let inner_zip_file = fs::File::create(&inner_zip_path).map_err(|e| e.to_string())?;
        let mut inner_zip = zip::ZipWriter::new(inner_zip_file);
        let game_data_files = get_project_files(project_path);
        for (file_path, zip_path) in game_data_files {
            if file_path.extension() == Some(std::ffi::OsStr::new("luau")) {
                // Compile into bytecode
                let script_content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
                let compiler = mlua::Compiler::new()
                    .set_optimization_level(2)
                    .set_type_info_level(1);
                let result = compiler.compile(script_content);
                match result {
                    Ok(bytecode) => {
                        add_file_content_to_zip(
                            &mut inner_zip,
                            &bytecode,
                            &zip_path,
                            SimpleFileOptions::default(),
                        )
                        .map_err(|e| e.to_string())?;
                    }
                    Err(err) => {
                        println!("Failed to compile {}: {}", file_path.display(), err);
                        add_file_to_zip_from_path(
                            &mut inner_zip,
                            &file_path,
                            &zip_path,
                            false,
                            false,
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }
            } else {
                add_file_to_zip_from_path(&mut inner_zip, &file_path, &zip_path, false, false)
                    .map_err(|e| e.to_string())?;
            }
        }
        inner_zip.finish().map_err(|e| e.to_string())?;

        add_file_to_zip_from_path(
            &mut zip,
            &inner_zip_path,
            "bundle.vecta",
            false,
            false, // avoid double compression
        )
        .map_err(|e| e.to_string())?;
        let _ = fs::remove_file(&inner_zip_path);
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(output_path)
}
