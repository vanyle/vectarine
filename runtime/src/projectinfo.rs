use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectInfo {
    pub title: String,
    pub main_script_path: String,
    pub logo_path: String,
    pub description: String,
    pub tags: Vec<String>,
    pub loading_animation: String,
    pub default_screen_width: u32,
    pub default_screen_height: u32,
}

pub fn get_project_info(project_manifest_content: &str) -> Result<ProjectInfo> {
    let r = toml::from_str::<ProjectInfo>(project_manifest_content);
    if let Ok(r) = r {
        return Ok(r);
    }
    let manifest = project_manifest_content.parse::<toml::Table>()?;

    let get_str_or_default = |key: &str, default: &str| {
        manifest
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    };
    let get_u32_or_default = |key: &str, default: u32| {
        manifest
            .get(key)
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(default)
    };
    let tags = manifest.get("tags").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    });

    Ok(ProjectInfo {
        title: get_str_or_default("title", "Untitle Vectarine Game"),
        default_screen_width: get_u32_or_default("screen_width", 1200),
        default_screen_height: get_u32_or_default("screen_height", 800),
        description: get_str_or_default("description", ""),
        tags: tags.unwrap_or_else(std::vec::Vec::new),
        main_script_path: get_str_or_default("main_script_path", "scripts/game.luau"),
        logo_path: get_str_or_default("logo_path", "assets/logo.png"),
        loading_animation: get_str_or_default("loading_animation", "default"),
    })
}
