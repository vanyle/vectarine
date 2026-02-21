use regex::Regex;
use runtime::mlua;
use runtime::projectinfo::ProjectInfo;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use runtime::egui::TextBuffer;
use zip::write::SimpleFileOptions;

use crate::editorinterface::extra::geteditorpaths::{
    get_runtime_file_for_linux, get_runtime_file_for_macos, get_runtime_file_for_windows,
    get_runtime_file_paths_for_web,
};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum ExportPlatform {
    Windows,
    Linux,
    MacOS,
    Web,
}

pub fn export_project(
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
            let runtime_path = get_runtime_file_for_windows();
            if let Some(runtime_path) = runtime_path {
                add_file_to_zip_from_path(&mut zip, &runtime_path, "game.exe", true, false)
                    .map_err(|e| e.to_string())?;
            } else {
                return Err("Failed to locate runtime.exe".into());
            }
        }
        ExportPlatform::Linux => {
            let runtime_path = get_runtime_file_for_linux();
            if let Some(runtime_path) = runtime_path {
                add_file_to_zip_from_path(&mut zip, &runtime_path, "game", true, false)
                    .map_err(|e| e.to_string())?;
            } else {
                return Err("Failed to locate runtime executable".into());
            }
        }
        ExportPlatform::MacOS => {
            let runtime_path = get_runtime_file_for_macos();
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
        "release", "game", "output", "build", "debug", "export", "private", "luau-api",
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
