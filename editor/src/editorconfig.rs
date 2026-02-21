use runtime::game_resource::ResourceId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum WindowStyle {
    #[default]
    GameWithEditor,
    GameSeparateFromEditor,
}

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum TextEditor {
    // VSCode family
    #[default]
    VSCode,
    Antigravity,
    Cursor,
    // Non-VSCode based
    Zed,
    SublimeText,
    Vim,
    Neovim,
    Emacs,
}

impl std::fmt::Display for TextEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TextEditor::VSCode => "VSCode",
                TextEditor::Antigravity => "Antigravity",
                TextEditor::Cursor => "Cursor",
                TextEditor::Zed => "Zed",
                TextEditor::SublimeText => "Sublime Text",
                TextEditor::Vim => "Vim",
                TextEditor::Neovim => "Neovim",
                TextEditor::Emacs => "emacsclient",
            }
        )
    }
}

/// The editor config contains settings that are not specific to any project and are persisted across editor launches.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    pub is_console_shown: bool,
    pub is_resources_window_shown: bool,
    pub is_watcher_window_shown: bool,
    pub is_profiler_window_shown: bool,
    pub is_plugins_window_shown: bool,
    pub is_export_window_shown: bool,
    pub is_preferences_window_shown: bool,
    pub is_always_on_top: bool,
    pub is_editor_always_on_top: bool,
    pub debug_resource_shown: Option<ResourceId>,

    pub window_style: WindowStyle,

    pub opened_project_path: Option<String>,

    pub text_editor: Option<TextEditor>,
}
