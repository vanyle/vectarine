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
    projectinfo::{ProjectInfo, get_project_info},
};
use runtime::{io::localfs::LocalFileSystem, sdl2};

use crate::{
    editorinterface::extra::geteditorpaths::{PLUGIN_FILE_EXTENSION, does_path_end_with},
    luau,
    pluginsystem::{
        gameplugin::GamePlugin,
        hash::Hash,
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
                callback(Ok(Self {
                    project_path: project_path.to_path_buf(),
                    project_info,
                    game,
                    video,
                    window,
                    hook_timing,
                    hook_error,
                    plugins: Rc::new(RefCell::new(Vec::new())),
                }));
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
        // Filter out untrusted plugins
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
        let trusted_plugins_hashes = trusted_plugins
            .iter()
            .map(|plugin| plugin.hash)
            .collect::<HashSet<Hash>>();

        let trusted_plugin_paths_of_game = plugin_files.filter_map(|path| {
            let hash = Hash::from_path(&path)?;
            if !trusted_plugins_hashes.contains(&hash) {
                return None;
            }
            Some(path)
        });

        // Unpack the dynamic libraries needed and delete the other dynamic libraries
        let game_plugins = trusted_plugin_paths_of_game
            .filter_map(|path| GamePlugin::from_path(&path, trusted_plugins))
            .collect::<Vec<GamePlugin>>();

        let trusted_dynamic_library_paths = game_plugins
            .iter()
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

        // Sync the Lua API folder
        for game_plugin in &game_plugins {
            let Some(trusted_plugin) = &game_plugin.trusted_plugin else {
                continue;
            };
            let dest = luau_api_folder.join(format!("{}.luau", trusted_plugin.name.clone()));
            let _ = trusted_plugin.try_copy_lua_api(&dest);
        }

        // Build the Vec<GamePlugin>
        self.plugins.replace(game_plugins);
    }

    pub fn add_plugin(&self, plugin: TrustedPlugin) {
        let Some(project_folder) = self.project_folder() else {
            return;
        };
        let project_plugins_folder = project_folder.join("plugins");
        let luau_api_folder = project_folder.join("luau-api");
        let Some(plugin_name) = plugin.path.file_name() else {
            return;
        };
        let _ = fs::create_dir_all(&project_plugins_folder);
        let _ = fs::copy(&plugin.path, project_plugins_folder.join(plugin_name));
        // Also copy the Luau API if it exists
        let _ = plugin.try_copy_lua_api(&luau_api_folder);
    }
}
