use std::path::PathBuf;
use std::{fs, path::Path};

use runtime::anyhow;
use runtime::{projectinfo::ProjectInfo, toml};

use crate::project::copydirall::copy_dir_all;
use crate::project::geteditorpaths::get_luau_api_path;

static DEFAULT_CODE: &str = "const debug = require('@vectarine/debug')
const graphics = require('@vectarine/graphics')
const vec4 = require('@vectarine/vec4')
const vec = require('@vectarine/vec')

-- Need help to get started?
-- Read: https://github.com/vanyle/vectarine/blob/main/docs/user-manual.md
-- The manual is available offline in the Help menu.

debug.print(\"Loaded.\")

function Update(deltaTime: number)
    graphics.clear(vec4.WHITE)
    if graphics.drawSplashScreenIfNeeded({
        -- Put the resources that need to be loaded here.
    }, \"Loading\") then
        return
    end
    graphics.drawSplashScreen(\"Empty game\", 0.0)
    debug.fprint(\"Rendered in \", deltaTime, \"sec\")
end
";

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

static DEFAULT_VSCODE_SETTINGS: &str = r#"{
	"luau-lsp.platform.type": "standard",
	"luau-lsp.sourcemap.enabled": false,
	"luau-lsp.sourcemap.autogenerate": false,
    "[luau]": {
		"editor.defaultFormatter": "JohnnyMorganz.stylua",
		"editor.formatOnSave": true,
	},
}"#;

fn copy_default_luau_api(project_folder: &Path) -> Result<(), std::io::Error> {
    let luau_api_path = project_folder.join("luau-api");
    let reference_luau_api_path = get_luau_api_path();
    copy_dir_all(reference_luau_api_path, luau_api_path)
}

pub fn create_game_and_get_path(game_name: &str, game_path: &Path) -> anyhow::Result<PathBuf> {
    let project_folder = game_path.join(game_name);
    let project_file_path = project_folder.join("game.vecta");
    let vscode_settings_path = project_folder.join(".vscode/settings.json");
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
    setup_failed = setup_failed.or(fs::create_dir_all(
        vscode_settings_path
            .parent()
            .expect("The parent of .vscode/settings.json should exist"),
    )
    .err());
    setup_failed = setup_failed.or(fs::write(&vscode_settings_path, DEFAULT_VSCODE_SETTINGS).err());

    if let Some(setup_failed) = setup_failed {
        return Err(anyhow::anyhow!(
            "Unable to create a project at the provided location: {}",
            setup_failed
        ));
    }

    Ok(project_file_path)
}
