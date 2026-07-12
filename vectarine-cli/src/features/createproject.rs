use std::path::Path;

use runtime::anyhow;
use std::path::PathBuf;

use crate::project::createproject::create_game_and_get_path;

pub fn create_project(project_path: &Path, game_name: &str) -> anyhow::Result<PathBuf> {
    create_game_and_get_path(game_name, project_path)
}
