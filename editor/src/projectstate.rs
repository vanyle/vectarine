use std::{
    cell::RefCell,
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Instant,
};

use runtime::{
    anyhow::{self},
    game::Game,
    io::fs::ReadOnlyFileSystem,
    lua_env::BUILT_IN_MODULES,
    projectinfo::{ProjectInfo, get_project_info},
};
use runtime::{io::localfs::LocalFileSystem, sdl2};

use crate::{
    editorinterface::extra::geteditorpaths::{
        PLUGIN_FILE_EXTENSION, does_path_end_with, get_luau_api_path,
    },
    luau,
    pluginsystem::{
        gameplugin::GamePlugin,
        trustedplugin::{TrustedPlugin, is_dynamic_library_file},
    },
};

pub struct ProjectState {
    /// Path to the .vecta file (the manifest) of the project
    pub project_path: PathBuf,
    pub project_info: ProjectInfo,
    pub game: Game,
    pub video: Rc<RefCell<sdl2::VideoSubsystem>>,
    pub window: Rc<RefCell<sdl2::video::Window>>,
    pub hook_timing: Rc<RefCell<Option<Instant>>>,
    pub hook_error: Rc<RefCell<Option<luau::InfiniteLoopError>>>,
    pub plugins: Rc<RefCell<Vec<GamePlugin>>>,
}

impl ProjectState {
    pub fn reload(&mut self) {
        let gl = self.game.gl.clone();
        Game::from_project(
            &self.project_path,
            &self.project_info,
            Box::new(LocalFileSystem),
            gl,
            &self.video,
            &self.window,
            |result| {
                let Ok(game) = result else {
                    return;
                };
                let (hook_timing, hook_error) = luau::setup_luau_hooks(&game.lua_env.lua);
                self.hook_timing = hook_timing;
                self.hook_error = hook_error;
                self.game = game;
            },
        );
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new<F>(
        project_path: &Path,
        file_system: Box<dyn ReadOnlyFileSystem>,
        gl: Arc<glow::Context>,
        video: Rc<RefCell<sdl2::VideoSubsystem>>,
        window: Rc<RefCell<sdl2::video::Window>>,
        trusted_plugins: &[TrustedPlugin],
        callback: F,
    ) where
        F: FnOnce(anyhow::Result<Self>),
    {
        let Ok(project_manifest_content) = fs::read_to_string(project_path) else {
            callback(Err(anyhow::anyhow!(
                "Failed to read the project manifest at {:?}",
                project_path
            )));
            return;
        };

        let Ok(project_info) = get_project_info(&project_manifest_content) else {
            callback(Err(anyhow::anyhow!(
                "Failed to parse the project manifest at {:?}",
                project_path
            )));
            return;
        };

        Game::from_project(
            project_path,
            &project_info.clone(),
            file_system,
            gl,
            &video.clone(),
            &window.clone(),
            move |result| {
                let Ok(game) = result else {
                    callback(Err(anyhow::anyhow!(
                        "Failed to load the game project at {:?}",
                        project_path
                    )));
                    return;
                };
                let (hook_timing, hook_error) = luau::setup_luau_hooks(&game.lua_env.lua);
                let result = Self {
                    project_path: project_path.to_path_buf(),
                    project_info,
                    game,
                    video,
                    window,
                    hook_timing,
                    hook_error,
                    plugins: Rc::new(RefCell::new(Vec::new())),
                };
                result.refresh_plugin_list(trusted_plugins);
                callback(Ok(result));
            },
        );
    }

    pub fn project_folder(&self) -> Option<&Path> {
        self.project_path.parent()
    }

    pub fn project_plugins_folder(&self) -> Option<PathBuf> {
        self.project_folder().map(|folder| folder.join("plugins"))
    }

    pub fn refresh_plugin_list(&self, trusted_plugins: &[TrustedPlugin]) {
        self.plugins.borrow_mut().clear();
        let Some(project_folder) = self.project_folder() else {
            return;
        };
        let project_plugins_folder = project_folder.join("plugins");
        let luau_api_folder = project_folder.join("luau-api");

        // Read the files in the folder
        let Ok(files) = fs::read_dir(&project_plugins_folder) else {
            return;
        };

        let plugin_files = files.filter_map(|file| {
            let Ok(file) = file else {
                return None;
            };
            let path = file.path();
            if !does_path_end_with(&path, PLUGIN_FILE_EXTENSION) {
                return None;
            }
            Some(path)
        });

        let game_plugins = plugin_files
            .filter_map(|path| GamePlugin::from_path(&path, trusted_plugins))
            .collect::<Vec<GamePlugin>>();

        // Filter out untrusted plugins
        let trusted_dynamic_library_paths = game_plugins
            .iter()
            .filter(|plugin| plugin.trusted_plugin.is_some()) // take only trusted plugins
            .map(|plugin| plugin.dynamic_library_path.clone())
            .collect::<HashSet<PathBuf>>();

        let Ok(files) = fs::read_dir(&project_plugins_folder) else {
            return;
        };
        for file in files {
            let Ok(file) = file else {
                continue;
            };
            let path = file.path();
            if !is_dynamic_library_file(&path) {
                continue;
            }
            // Only keep trusted dynamic libraries
            if !trusted_dynamic_library_paths.contains(&path) {
                let _ = fs::remove_file(&path);
            }
        }

        // Extract dynamic libraries of trusted plugins
        for plugin in &game_plugins {
            let Some(trusted_plugin) = &plugin.trusted_plugin else {
                continue;
            };
            let _ = trusted_plugin.try_copy_dynamic_library(&plugin.dynamic_library_path);
        }

        // Sync the Lua API folder
        if !luau_api_folder.is_dir() && luau_api_folder.exists() {
            let _ = fs::remove_file(&luau_api_folder);
        }
        if !luau_api_folder.exists() {
            let _ = fs::create_dir(&luau_api_folder);
        }
        let mut known_luau_files = BUILT_IN_MODULES
            .iter()
            .map(|module| format!("{}.luau", module))
            .collect::<HashSet<String>>();

        // Add the Lua API files of the plugins
        for game_plugin in &game_plugins {
            let Some(trusted_plugin) = &game_plugin.trusted_plugin else {
                continue;
            };
            let name = format!("{}.luau", trusted_plugin.name.clone());
            let dest = luau_api_folder.join(name.clone());
            known_luau_files.insert(name);
            let _ = trusted_plugin.try_copy_lua_api(&dest);
        }

        // Remove unknown Lua API files
        if let Ok(files) = fs::read_dir(&luau_api_folder) {
            for file in files {
                let Ok(file) = file else {
                    continue;
                };
                let filename = file
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if !known_luau_files.contains(&filename) {
                    let _ = fs::remove_file(file.path());
                }
            }
        }

        // Add the built-in modules
        let luau_editor_path = get_luau_api_path();
        BUILT_IN_MODULES.iter().for_each(|module_name| {
            let src = luau_editor_path.join(format!("{}.luau", module_name));
            let dest = luau_api_folder.join(format!("{}.luau", module_name));
            if !src.exists() {
                // avoid unnecessary file writing.
                let _ = fs::copy(src, dest);
            }
        });

        // Build the Vec<GamePlugin>
        self.plugins.replace(game_plugins);
        self.save_project_info();
    }

    /// Add a plugin to the project.
    /// A refresh of the plugin list is needed after that.
    pub fn add_plugin(&self, plugin: TrustedPlugin) {
        let Some(project_folder) = self.project_folder() else {
            return;
        };
        let project_plugins_folder = project_folder.join("plugins");
        let Some(plugin_name) = plugin.path.file_name() else {
            return;
        };
        let _ = fs::create_dir_all(&project_plugins_folder);
        let _ = fs::copy(&plugin.path, project_plugins_folder.join(plugin_name));
    }

    pub fn update_plugins_in_project_info(&mut self) {
        self.project_info.plugins = self
            .plugins
            .borrow()
            .iter()
            .filter_map(|plugin| {
                plugin.trusted_plugin.as_ref()?; // only keep trusted plugins.
                let filename = plugin.dynamic_library_path.file_prefix()?;
                Some(filename.to_string_lossy().to_string())
            })
            .collect();
    }

    /// Save the project info to the project manifest while trying to preserve comments general order of keys.
    pub fn save_project_info(&self) {
        let Ok(current_project_info) = fs::read_to_string(&self.project_path) else {
            self.save_project_info_by_overwriting();
            return;
        };
        let Ok(mut document) = current_project_info.parse::<toml_edit::DocumentMut>() else {
            self.save_project_info_by_overwriting();
            return;
        };
        let toml_string = vectarine_plugin_sdk::toml::to_string(&self.project_info)
            .expect("Unable to serialize the ProjectInfo type to toml");
        let target_document = toml_string
            .parse::<toml_edit::DocumentMut>()
            .expect("Unable to parse the toml string generated by toml");
        for (key, value) in target_document.iter() {
            document[key] = value.clone();
        }
        let toml_string = document.to_string();
        let _ = fs::write(&self.project_path, toml_string);
    }

    /// Save the project info while erasing the existing file and its fields.
    fn save_project_info_by_overwriting(&self) {
        let toml_string = vectarine_plugin_sdk::toml::to_string(&self.project_info)
            .expect("Unable to serialize the ProjectInfo type to toml");
        let _ = fs::write(&self.project_path, toml_string);
    }
}
