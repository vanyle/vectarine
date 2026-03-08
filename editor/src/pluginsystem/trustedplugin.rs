use std::{
    collections::HashSet,
    fs,
    io::{self, Read},
    path::Path,
};

use runtime::{
    anyhow::{self, bail},
    native_plugin::DYNAMIC_LIB_SUFFIXES,
    toml,
};
use serde::Deserialize;

use crate::{
    editorinterface::extra::geteditorpaths::{
        PLUGIN_FILE_EXTENSION, does_path_end_with, get_editor_plugins_path,
    },
    export::exportproject::ExportPlatform,
    pluginsystem::hash::Hash,
};

/// A trusted plugin is a plugin in the list of plugins that the editor knows about.
/// These are not loaded and are specific to the editor, not to a given game / project.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TrustedPlugin {
    pub name: String,
    pub version: u64,
    pub path: std::path::PathBuf,
    has_lua_api: bool,
    pub supported_platforms: HashSet<ExportPlatform>,
    pub hash: Hash,
    pub url: String,
    pub description: String,
}

pub struct MalformedPlugin {
    pub path: std::path::PathBuf,
    pub error: String,
}

// The UI will display plugin entries
pub enum PluginEntry {
    Trusted(TrustedPlugin),
    Malformed(MalformedPlugin),
}

impl TrustedPlugin {
    /// Tries to copy the lua api of the plugin to the given destination.
    /// If the destination path already exists, it will be overwritten.
    /// If the plugin does not have a lua api, it will do nothing.
    pub fn try_copy_lua_api(&self, dest: &Path) -> anyhow::Result<()> {
        copy_file_from_vectaplugin(&self.path, "plugin.luau", dest)
    }

    pub fn try_copy_dynamic_library(&self, dest: &Path) -> anyhow::Result<()> {
        let platform_file = get_platform_file_for_me();
        copy_file_from_vectaplugin(&self.path, platform_file, dest)
    }

    /// Checks if the file containing the plugin is still valid.
    pub fn is_still_valid(&self) -> bool {
        let Ok(file) = fs::File::open(&self.path) else {
            return false;
        };
        let mut file_reader = io::BufReader::new(file);
        let hash = Hash::from_file(&mut file_reader);
        match hash {
            Some(hash) => self.hash == hash,
            None => false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PluginTomlManifest {
    name: String,
    version: u64,
    url: String,
    description: String,
    // Maybe one day, this will contain some kind of signature to verify authorship. For now, url + hash is enough.
    // The editor can display the hash, the url can display the hash and the human can check that they match
}

pub fn load_plugins() -> Vec<PluginEntry> {
    let plugin_library_path = get_editor_plugins_path();
    if !plugin_library_path.exists() {
        let _ = fs::create_dir_all(&plugin_library_path);
        return vec![];
    }
    let Ok(entries) = fs::read_dir(&plugin_library_path) else {
        return vec![];
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !does_path_end_with(&entry.path(), PLUGIN_FILE_EXTENSION) {
                return None;
            }
            Some(entry)
        })
        .map(|entry| match load_trusted_plugin(&entry.path()) {
            Ok(plugin) => PluginEntry::Trusted(plugin),
            Err(err) => PluginEntry::Malformed(MalformedPlugin {
                path: entry.path(),
                error: err.to_string(),
            }),
        })
        .collect::<Vec<_>>()
}

static PLATFORM_FILES: [(ExportPlatform, &str); 4] = [
    (ExportPlatform::Windows, "windows/plugin.dll"),
    (ExportPlatform::Linux, "linux/plugin.so"),
    (ExportPlatform::MacOS, "macos/plugin.dylib"),
    (ExportPlatform::Web, "web/plugin.wasm"),
];

pub fn get_platform_file_for_me() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows/plugin.dll"
    }
    #[cfg(target_os = "linux")]
    {
        "linux/plugin.so"
    }
    #[cfg(target_os = "macos")]
    {
        "macos/plugin.dylib"
    }
    #[cfg(target_arch = "wasm32")]
    {
        "web/plugin.wasm"
    }
}

fn copy_file_from_vectaplugin(zip_path: &Path, file_name: &str, dest: &Path) -> anyhow::Result<()> {
    let Ok(file) = fs::File::open(zip_path) else {
        bail!("The vectaplugin file cannot be read");
    };
    let Ok(mut zip_archive) = zip::ZipArchive::new(file) else {
        bail!("It is not a valid {} file", PLUGIN_FILE_EXTENSION);
    };
    let Ok(mut file) = zip_archive.by_name(file_name) else {
        bail!("It does not contain a {} file", file_name);
    };
    let mut dest_writer = fs::File::create(dest)?;
    io::copy(&mut file, &mut dest_writer)?;
    Ok(())
}

fn load_trusted_plugin(path: &Path) -> anyhow::Result<TrustedPlugin> {
    if !does_path_end_with(path, PLUGIN_FILE_EXTENSION) {
        bail!("The file does not end with {}", PLUGIN_FILE_EXTENSION);
    }
    let Ok(file) = fs::File::open(path) else {
        bail!("The file cannot be read");
    };
    let mut file_reader = io::BufReader::new(file);
    let hash =
        Hash::from_file(&mut file_reader).ok_or(anyhow::anyhow!("Failed to compute hash"))?;
    let Ok(file) = fs::File::open(path) else {
        bail!("The file cannot be read, again?"); // Indicate some weird race condition or somebody messing with us.
    };
    let Ok(mut zip_archive) = zip::ZipArchive::new(file) else {
        bail!(
            "It is not a valid {} file. The file might be corrupted",
            PLUGIN_FILE_EXTENSION
        );
    };

    let supported_platforms = PLATFORM_FILES
        .iter()
        .filter(|(_, file)| zip_archive.by_name(file).is_ok())
        .map(|(platform, _)| *platform)
        .collect::<HashSet<ExportPlatform>>();

    let has_lua_api = zip_archive.by_name("plugin.luau").is_ok();

    let Ok(mut manifest) = zip_archive.by_name("manifest.toml") else {
        bail!("It does not contain a plugin manifest");
    };
    let mut buf = String::new();
    manifest.read_to_string(&mut buf)?;
    let manifest = match toml::from_str::<PluginTomlManifest>(&buf) {
        Ok(toml) => toml,
        Err(err) => bail!("It does not contain a valid plugin manifest, {}", err),
    };

    Ok(TrustedPlugin {
        hash,
        name: manifest.name,
        version: manifest.version,
        path: path.to_path_buf(),
        has_lua_api,
        supported_platforms,
        url: manifest.url,
        description: manifest.description,
    })
}

pub fn get_hash_of_file_in_zip(zip_path: &Path, file_name: &str) -> Option<Hash> {
    let Ok(file) = fs::File::open(zip_path) else {
        return None;
    };
    let Ok(mut zip_archive) = zip::ZipArchive::new(file) else {
        return None;
    };
    let Ok(file) = zip_archive.by_name(file_name) else {
        return None;
    };
    let mut file_reader = io::BufReader::new(file);
    Hash::from_file(&mut file_reader)
}

pub fn is_dynamic_library_file(path: &Path) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };
    DYNAMIC_LIB_SUFFIXES.contains(&ext.to_string_lossy().as_ref())
}
