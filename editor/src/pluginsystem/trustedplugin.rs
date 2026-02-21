use std::{collections::HashSet, fs};

use crate::{
    editorinterface::extra::geteditorpaths::get_editor_plugins_path,
    export::exportproject::ExportPlatform, pluginsystem::hash::Hash,
};

/// A trusted plugin is a plugin in the list of plugins that the editor knows about.
/// These are not loaded and are specific to the editor, not to a given game / project.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TrustedPlugin {
    pub name: String,
    pub version: String,
    pub path: std::path::PathBuf,
    pub lua_api_path: Option<std::path::PathBuf>,
    pub supported_platforms: HashSet<ExportPlatform>,
    pub hash: Hash,
}

pub fn load_trusted_plugins() -> Vec<TrustedPlugin> {
    let plugin_library_path = get_editor_plugins_path();
    if !plugin_library_path.exists() {
        let _ = fs::create_dir_all(&plugin_library_path);
        return vec![];
    }
    vec![]
}
