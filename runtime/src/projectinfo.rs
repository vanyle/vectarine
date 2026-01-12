use vectarine_plugin_sdk::anyhow::Result;
use vectarine_plugin_sdk::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "vectarine_plugin_sdk::serde")]
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

impl Default for ProjectInfo {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            main_script_path: "scripts/game.luau".to_string(),
            logo_path: "".to_string(),
            description: "".to_string(),
            tags: vec![],
            default_screen_width: 800,
            default_screen_height: 600,
            loading_animation: "pixel".to_string(),
        }
    }
}

pub fn get_project_info(project_manifest_content: &str) -> Result<ProjectInfo> {
    let r = vectarine_plugin_sdk::toml::from_str::<ProjectInfo>(project_manifest_content);
    if let Ok(r) = r {
        return Ok(r);
    }
    let manifest = project_manifest_content.parse::<vectarine_plugin_sdk::toml::Table>()?;

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
