use std::fs;
use std::path::{Path, PathBuf};

use runtime::anyhow::{self, bail};

use crate::pluginsystem::hash::Hash;
use crate::pluginsystem::trustedplugin::{
    TrustedPlugin, get_hash_of_file_in_zip, get_platform_file_for_me,
};

/// Represents a plugin loaded into a game. It can be trusted or not.
#[derive(Debug)]
pub struct GamePlugin {
    pub path: PathBuf,
    pub trusted_plugin: Option<TrustedPlugin>,
    pub dynamic_library_path: PathBuf,
    pub dynamic_library_hash: Option<Hash>,
}

impl GamePlugin {
    pub fn from_path(path: &Path, trusted_plugins: &[TrustedPlugin]) -> Option<Self> {
        let Some(hash) = Hash::from_path(path) else {
            return None;
        };
        let trusted_plugin = trusted_plugins.iter().find(|plugin| plugin.hash == hash);
        let dynamic_library_path = get_associated_dynamic_library_path(path);
        let path_in_zip = get_platform_file_for_me();
        let hash = get_hash_of_file_in_zip(path, path_in_zip);

        Some(Self {
            path: path.to_path_buf(),
            trusted_plugin: trusted_plugin.cloned(),
            dynamic_library_path,
            dynamic_library_hash: hash,
        })
    }

    pub fn create_dynamic_library_file_if_needed(&self) -> anyhow::Result<()> {
        let Some(expected_hash) = &self.dynamic_library_hash else {
            bail!("The plugin does not support the current platform");
        };

        if self.dynamic_library_path.exists() {
            if let Some(hash) = Hash::from_path(&self.dynamic_library_path) {
                // Hashes are matching, everything is OK, no need to recreate the file.
                if hash == *expected_hash {
                    return Ok(());
                }
            }
            // File is corrupted, or the plugin was updated or the project loaded was malicious.
            let _ = fs::remove_file(&self.dynamic_library_path);
        }

        let Some(trusted_plugin) = &self.trusted_plugin else {
            bail!("The plugin is not trusted");
        };
        trusted_plugin.try_copy_dynamic_library(&self.dynamic_library_path)
    }
}

fn get_associated_dynamic_library_path(path: &Path) -> PathBuf {
    let mut dynamic_library_path = path.to_path_buf();
    dynamic_library_path.set_extension(runtime::native_plugin::get_dynamic_lib_suffix());
    dynamic_library_path
}
