use std::{fs, path::Path};

use runtime::{io::localfs::LocalFileSystem, projectinfo::ProjectInfo, toml};

use crate::{
    copydirall::copy_dir_all,
    editorinterface::{EditorState, geteditorpaths::get_luau_api_path},
};

static DEFAULT_CODE: &str = "local Debug = require('@vectarine/debug')
local Graphics = require('@vectarine/graphics')
local Vec4 = require('@vectarine/vec4')
local Vec = require('@vectarine/vec')
Debug.print(\"Loaded.\")
function Update(deltaTime: number)
    Graphics.clear(Vec4.WHITE)
    Graphics.drawSplashScreen(\"Empty game\", 0.0)
    Debug.fprint(\"Rendered in \", deltaTime, \"sec\")
end";

static DEFAULT_LUAURC: &str = r#"{
	"languageMode": "strict",
	"lintErrors": false,
	"lint": {
		"FunctionUnused": false
	},
	"aliases": {
		"vectarine": "luau-api"
	}
}"#;

fn copy_default_luau_api(project_folder: &Path) -> Result<(), std::io::Error> {
    let luau_api_path = project_folder.join("luau-api");
    let reference_luau_api_path = get_luau_api_path();
    copy_dir_all(reference_luau_api_path, luau_api_path)
}

pub fn create_game_and_open_it(state: &mut EditorState, game_name: &str, game_path: &Path) {
    let project_folder = game_path.join(game_name);
    let project_file_path = project_folder.join("game.vecta");
    let script_folder = project_folder.join("scripts");
    let project_info = ProjectInfo {
        title: game_name.to_string(),
        ..ProjectInfo::default()
    };

    let main_script_path = project_folder.join(&project_info.main_script_path);
    let mut setup_failed = None;

    // By default, a project is:
    // - a game.vecta file
    // - a scripts/game.luau file
    // - luau-api folder with a copy of the scripts
    // - a .luaurc file
    setup_failed = setup_failed.or(fs::create_dir_all(script_folder).err());
    {
        let serialized = toml::to_string(&project_info).unwrap_or_default();
        setup_failed = setup_failed.or(fs::write(&project_file_path, serialized).err());
    }

    setup_failed = setup_failed.or(fs::write(&main_script_path, DEFAULT_CODE).err());
    setup_failed = setup_failed.or(copy_default_luau_api(&project_folder).err());
    setup_failed = setup_failed.or(fs::write(project_folder.join(".luaurc"), DEFAULT_LUAURC).err());

    if let Some(setup_failed) = setup_failed {
        println!(
            "Unable to create a project at the provided location: {}",
            setup_failed
        );
        return;
    }

    state.load_project(Box::new(LocalFileSystem), &project_file_path, |result| {
        if let Err(e) = result {
            // TODO: show error in GUI
            println!("Failed to load project: {e}");
        }
    });
}
