use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use runtime::{
    anyhow::{self, Result},
    game::Game,
    game_resource::script_resource::ScriptResource,
    graphics::batchdraw::BatchDraw2d,
    lua_env::LuaEnvironment,
};

pub struct ProjectState {
    pub project_path: PathBuf,
    pub game: Game,
}

impl ProjectState {
    pub fn new(project_path: &Path, gl: Arc<glow::Context>) -> Result<Self> {
        let project_dir = project_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid project path"))?;

        let batch = BatchDraw2d::new(&gl).unwrap();
        let lua_env = LuaEnvironment::new(batch, project_dir);
        let game = Game::new(&gl, lua_env);

        let path = Path::new("scripts/game.luau");
        game.lua_env.resources.load_resource::<ScriptResource>(
            path,
            gl.clone(),
            game.lua_env.lua.clone(),
            game.lua_env.default_events.resource_loaded_event,
        );

        Ok(Self {
            project_path: project_path.to_path_buf(),
            game,
        })
    }
}
