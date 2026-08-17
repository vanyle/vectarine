use std::path::Path;

use runtime::anyhow;
use std::path::PathBuf;

use crate::project::createproject::{
    ProjectCreationOptions, StartingProjectTemplate, create_game_and_get_path,
};

pub fn create_project(project_path: &Path, game_name: &str) -> anyhow::Result<PathBuf> {
    let options = ProjectCreationOptions {
        name: game_name.to_string(),
        project_location: project_path.to_path_buf(),
        init_git_repo: true,
        init_vs_settings: true,
        template: StartingProjectTemplate::FromScratch,
    };
    create_game_and_get_path(&options)
}
