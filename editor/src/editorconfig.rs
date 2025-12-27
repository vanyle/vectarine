use runtime::game_resource::ResourceId;
use serde::{Deserialize, Serialize};

pub const EDITOR_CONFIG_FILE: &str = "vectarine-config.toml";

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum WindowStyle {
    #[default]
    GameWithEditor,
    GameSeparateFromEditor,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    pub is_console_shown: bool,
    pub is_resources_window_shown: bool,
    pub is_watcher_window_shown: bool,
    pub is_profiler_window_shown: bool,
    pub is_export_window_shown: bool,
    pub is_always_on_top: bool,
    pub debug_resource_shown: Option<ResourceId>,

    pub window_style: WindowStyle,

    pub opened_project_path: Option<String>,
}
