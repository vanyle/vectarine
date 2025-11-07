use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use runtime::{
    anyhow::{self},
    game::Game,
    io::fs::ReadOnlyFileSystem,
    projectinfo::{ProjectInfo, get_project_info},
};
use runtime::{io::localfs::LocalFileSystem, sdl2};

pub struct ProjectState {
    pub project_path: PathBuf,
    pub project_info: ProjectInfo,
    pub game: Game,
    pub video: Rc<RefCell<sdl2::VideoSubsystem>>,
    pub window: Rc<RefCell<sdl2::video::Window>>,
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
                callback(Ok(Self {
                    project_path: project_path.to_path_buf(),
                    project_info,
                    game,
                    video,
                    window,
                }));
            },
        );
    }
}
