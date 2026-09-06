use std::fmt::Display;
use std::path::PathBuf;
use std::{fs, path::Path};

use runtime::anyhow;
use runtime::{projectinfo::ProjectInfo, toml};
use std::process::Command;

use crate::buildinfo;
use crate::project::copydirall::copy_dir_all;
use crate::project::geteditorpaths::{get_gallery_path, get_luau_api_path};
use crate::project::new_project_strings::{
    DEFAULT_CODE, DEFAULT_GITIGNORE, DEFAULT_LUAURC, DEFAULT_VSCODE_SETTINGS,
};

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
    pub fn path(&self) -> Option<&'static str> {
        match self {
            StartingProjectTemplate::FromScratch => None,
            StartingProjectTemplate::Platformer => Some("Platformer"),
            StartingProjectTemplate::TopDownRPG => Some("Snake"), // Does not exist yet :'(
            StartingProjectTemplate::Sokoban => Some("Sokoban"),
            StartingProjectTemplate::AlienShooter => Some("AlienShooter"),
        }
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

fn copy_default_luau_api(project_folder: &Path) -> Result<(), std::io::Error> {
    let luau_api_path = project_folder.join("luau-api");
    let reference_luau_api_path = get_luau_api_path();
    copy_dir_all(reference_luau_api_path, luau_api_path)
}

pub fn create_game_and_get_path(options: &ProjectCreationOptions) -> anyhow::Result<PathBuf> {
    // By default, a project is:
    // - a game.vecta file
    // - scripts, a single "scripts/game.luau" file for the "from scratch" template, maybe more for other templates
    // - assets depending on the template choosen
    // - luau-api folder with a copy of the scripts
    // - a .luaurc file
    // - .vscode/settings.json if the user wants to init VS Code settings
    // - .gitignore and git related things if the user wants to init a git repo

    let project_folder = options.project_location.join(options.name.clone());
    let project_file_path = project_folder.join("game.vecta");

    let project_info = ProjectInfo {
        title: options.name.to_string(),
        vectarine_version: Some(buildinfo::get_version().to_string()),
        ..ProjectInfo::default()
    };

    let mut setup_failed = None;

    // Create the project folder.
    setup_failed = setup_failed.or(fs::create_dir_all(&project_folder).err());

    {
        // game.vecta
        let serialized = toml::to_string(&project_info).unwrap_or_default();
        setup_failed = setup_failed.or(fs::write(&project_file_path, serialized).err());
    }

    // scripts / template
    match options.template {
        StartingProjectTemplate::FromScratch => {
            let main_script_path = project_folder.join(&project_info.main_script_path);
            let script_folder = project_folder.join("scripts");
            setup_failed = setup_failed.or(fs::create_dir_all(script_folder).err());
            setup_failed = setup_failed.or(fs::write(&main_script_path, DEFAULT_CODE).err());
        }
        // If the template does not exist, the GUI should not allow the user to select the option.
        _ => {
            let template_path = get_template_path_to_copy(options.template).ok_or_else(|| {
                anyhow::anyhow!(
                    "The selected template ({}) does not exist.",
                    options.template
                )
            })?;
            setup_failed =
                setup_failed.or(copy_template_to_project(template_path, &project_folder).err());
        }
    }

    // luau-api and .luaurc
    setup_failed = setup_failed.or(copy_default_luau_api(&project_folder).err());
    setup_failed = setup_failed.or(fs::write(project_folder.join(".luaurc"), DEFAULT_LUAURC).err());

    if options.init_vs_settings {
        let vscode_settings_path = project_folder.join(".vscode/settings.json");
        setup_failed = setup_failed.or(fs::create_dir_all(
            vscode_settings_path
                .parent()
                .expect("The parent of .vscode/settings.json should exist"),
        )
        .err());
        setup_failed =
            setup_failed.or(fs::write(&vscode_settings_path, DEFAULT_VSCODE_SETTINGS).err());
    }

    if options.init_git_repo {
        let gitignore = project_folder.join(".gitignore");
        setup_failed = setup_failed.or(fs::write(&gitignore, DEFAULT_GITIGNORE).err());

        Command::new("git")
            .arg("init")
            .arg(&project_folder)
            .output()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to initialize git repository in {}: {}",
                    project_folder.display(),
                    e
                )
            })?;
    }

    if let Some(setup_failed) = setup_failed {
        return Err(anyhow::anyhow!(
            "Unable to create a project at the provided location: {}",
            setup_failed
        ));
    }

    Ok(project_file_path)
}

pub fn get_template_path_to_copy(template: StartingProjectTemplate) -> Option<PathBuf> {
    if matches!(template, StartingProjectTemplate::FromScratch) {
        return None;
    }
    let path = template
        .path()
        .expect("All templates have a path except for 'from scratch'");
    let template_path = get_gallery_path().join(path);
    Some(template_path)
}

pub fn is_template_available_for_project_creation(template: StartingProjectTemplate) -> bool {
    if matches!(template, StartingProjectTemplate::FromScratch) {
        return true;
    }
    let path = template
        .path()
        .expect("All templates have a path except for 'from scratch'");
    let template_path = get_gallery_path().join(path);
    template_path.exists() && template_path.is_dir()
}

pub fn copy_template_to_project(
    template_path: impl AsRef<Path>,
    project_path: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    // When copying a template, we copy:
    // - Only audio, levels, textures, scripts, shaders and fonts folders
    // - We do not copy game.vecta, .luaurc, .gitignore, etc.
    static AVOIDED_FILES: &[&str] = &[
        "",
        "game.vecta",
        ".luaurc",
        ".gitignore",
        ".vscode",
        "plugins",
        "luau-api",
        ".git",
        ".DS_Store",
    ];

    fs::create_dir_all(&project_path)?;
    for entry in fs::read_dir(template_path)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if AVOIDED_FILES.contains(&entry.file_name().to_string_lossy().as_ref()) {
            continue;
        }
        if ty.is_dir() {
            copy_template_to_project(entry.path(), project_path.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), project_path.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
