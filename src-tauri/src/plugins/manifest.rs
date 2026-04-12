use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PluginCategory {
    Viewer,
    Column,
    Action,
    Panel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub category: PluginCategory,
    #[serde(default)]
    pub file_extensions: Vec<String>,
    pub entrypoint: String,

    // Column-specific
    #[serde(default)]
    pub column_name: Option<String>,
    #[serde(default)]
    pub column_width: Option<u32>,

    // Action-specific
    #[serde(default)]
    pub keybinding: Option<String>,
    #[serde(default)]
    pub menu_label: Option<String>,

    // Panel-specific
    #[serde(default)]
    pub protocol: Option<String>,

    #[serde(default)]
    pub min_app_version: Option<String>,
    #[serde(default = "bool_true")]
    pub enabled: bool,
}

fn bool_true() -> bool {
    true
}

/// A manifest paired with its resolved filesystem paths.
#[derive(Debug, Clone, Serialize)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    /// Absolute path to the plugin's directory, e.g. ~/.nyx/plugins/com.example.foo/
    pub plugin_dir: String,
    /// Absolute path to the entrypoint (HTML file or executable).
    pub entrypoint_path: String,
}

/// Load and parse a plugin.json from the given directory.
/// Returns None if the file is missing, malformed, or disabled.
pub fn load_manifest(plugin_dir: &Path) -> Option<LoadedPlugin> {
    let manifest_path = plugin_dir.join("plugin.json");
    let data = std::fs::read_to_string(&manifest_path).ok()?;
    let manifest: PluginManifest = serde_json::from_str(&data).ok()?;
    let entrypoint_path = plugin_dir
        .join(&manifest.entrypoint)
        .to_string_lossy()
        .to_string();
    Some(LoadedPlugin {
        plugin_dir: plugin_dir.to_string_lossy().to_string(),
        entrypoint_path,
        manifest,
    })
}
