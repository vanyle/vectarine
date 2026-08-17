use std::fmt::Display;
use std::path::PathBuf;
use std::{fs, path::Path};

use runtime::anyhow;
use runtime::{projectinfo::ProjectInfo, toml};

use crate::buildinfo;
use crate::project::copydirall::copy_dir_all;
use crate::project::geteditorpaths::get_luau_api_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartingProjectTemplate {
    FromScratch,
    Platformer,
    TopDownRPG,
    Sokoban,
    AlienShooter,
}

impl StartingProjectTemplate {
    pub fn all_templates() -> Vec<StartingProjectTemplate> {
        vec![
            StartingProjectTemplate::FromScratch,
            StartingProjectTemplate::Platformer,
            StartingProjectTemplate::TopDownRPG,
            StartingProjectTemplate::Sokoban,
            StartingProjectTemplate::AlienShooter,
        ]
    }
    pub fn description(&self) -> &'static str {
        match self {
            StartingProjectTemplate::FromScratch => "A blank project",
            StartingProjectTemplate::Platformer => {
                "A platformer side-scroller with a player character, physics and a world loaded on demand."
            }
            StartingProjectTemplate::TopDownRPG => {
                "A top-down RPG with health, enemies and a tilemap-based world loaded on demand."
            }
            StartingProjectTemplate::Sokoban => {
                "A top-down Sokoban-style puzzle with undo/redo, pushable boxes and special items."
            }
            StartingProjectTemplate::AlienShooter => {
                "A shooter side-scroller with a ship, enemies, and projectiles."
            }
        }
    }
}

pub struct ProjectCreationOptions {
    pub template: StartingProjectTemplate,
    pub init_git_repo: bool,
    pub init_vs_settings: bool,
    pub name: String,
    pub project_location: PathBuf,
}

impl Display for StartingProjectTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartingProjectTemplate::FromScratch => write!(f, "From Scratch"),
            StartingProjectTemplate::Platformer => write!(f, "Platformer"),
            StartingProjectTemplate::TopDownRPG => write!(f, "Top-down RPG"),
            StartingProjectTemplate::Sokoban => write!(f, "Sokoban"),
            StartingProjectTemplate::AlienShooter => write!(f, "Alien Shooter"),
        }
    }
}

static DEFAULT_CODE: &str = "const debug = require(\"@vectarine/debug\")
const graphics = require(\"@vectarine/graphics\")
local persist = require(\"@vectarine/persist\")
local ui = require(\"@vectarine/ui\")
const vec4 = require(\"@vectarine/vec4\")
const vec = require(\"@vectarine/vec\")

-- Need help to get started?
-- Read: https://vectarineengine.com/guides/overview
-- The manual is available offline in the Help menu.

-- Print a message to the console (open the console with Ctrl+1 or from the Tools menu)
debug.print(\"Loaded.\")

-- Persist the fake timer across reloads so that it doesn't reset when you edit the code and reload the game.
local fakeLoadingTimer = persist.onReload({ value = 0.0 }, \"fakeLoadingTimer\")

local gettingStartedUi = ui.stack({ alignX = \"center\", alignY = \"center\" }, {
	ui.spacer(vec.V2(2, 2)),
	ui.column({ align = \"center\" }, {
		ui.text(
			\"Go to Tools > Resources (or press Ctrl+2) to open the resources panel\",
			{ color = vec4.WHITE, maxWidth = 1, align = \"left\" }
		),
		ui.spacer(vec.V2(0, 0.03)),
		ui.text(
			\"From there, click on game.luau to open it and get started!\",
			{ color = vec4.WHITE, maxWidth = 1, align = \"left\" }
		),
	}),
})

function Update(deltaTime: number)
	debug.fprint(\"Rendered in \", deltaTime, \"sec\")

	if
		graphics.drawSplashScreenIfNeeded({
			-- Put the resources that need to be loaded here.
		}, \"Loading\")
	then
		return
	end

	fakeLoadingTimer.value = fakeLoadingTimer.value + deltaTime
	if fakeLoadingTimer.value < 4.0 then
		graphics.drawSplashScreen(\"Pretending to load...\", fakeLoadingTimer.value / 4.0)
		return
	end

	gettingStartedUi:draw({})
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

pub fn create_game_and_get_path(options: &ProjectCreationOptions) -> anyhow::Result<PathBuf> {
    let project_folder = options.project_location.join(options.name.clone());
    let project_file_path = project_folder.join("game.vecta");
    let vscode_settings_path = project_folder.join(".vscode/settings.json");
    let script_folder = project_folder.join("scripts");
    let project_info = ProjectInfo {
        title: options.name.to_string(),
        vectarine_version: Some(buildinfo::get_version().to_string()),
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
