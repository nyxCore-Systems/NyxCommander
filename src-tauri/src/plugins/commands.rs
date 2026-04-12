use super::manifest::{LoadedPlugin, PluginCategory};
use super::registry::PluginState;
use crate::commands::FileEntry;
use serde::Serialize;
use serde_json::Value;
use std::io::Read;
use tauri::command;

// ─── Return types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    /// "viewer" | "column" | "action" | "panel"
    pub category: String,
    pub file_extensions: Vec<String>,
    /// Absolute path to the entrypoint (HTML file or executable).
    pub entrypoint_path: String,
    pub column_name: Option<String>,
    pub column_width: Option<u32>,
    pub keybinding: Option<String>,
    pub menu_label: Option<String>,
    pub protocol: Option<String>,
    pub enabled: bool,
}

impl From<&LoadedPlugin> for PluginInfo {
    fn from(p: &LoadedPlugin) -> Self {
        let category = match p.manifest.category {
            PluginCategory::Viewer => "viewer",
            PluginCategory::Column => "column",
            PluginCategory::Action => "action",
            PluginCategory::Panel => "panel",
        };
        PluginInfo {
            id: p.manifest.id.clone(),
            name: p.manifest.name.clone(),
            version: p.manifest.version.clone(),
            description: p.manifest.description.clone(),
            author: p.manifest.author.clone(),
            category: category.to_string(),
            file_extensions: p.manifest.file_extensions.clone(),
            entrypoint_path: p.entrypoint_path.clone(),
            column_name: p.manifest.column_name.clone(),
            column_width: p.manifest.column_width,
            keybinding: p.manifest.keybinding.clone(),
            menu_label: p.manifest.menu_label.clone(),
            protocol: p.manifest.protocol.clone(),
            enabled: p.manifest.enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnValue {
    pub path: String,
    pub value: String,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Return all loaded (enabled) plugins.
#[command]
pub async fn list_plugins(state: tauri::State<'_, PluginState>) -> Result<Vec<PluginInfo>, String> {
    let reg = state.0.lock().await;
    let mut plugins: Vec<PluginInfo> = reg.plugins.values().map(PluginInfo::from).collect();
    plugins.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(plugins)
}

/// Return the viewer plugin that handles the given file extension, if any.
#[command]
pub async fn get_viewer_for_file(
    ext: String,
    state: tauri::State<'_, PluginState>,
) -> Result<Option<PluginInfo>, String> {
    let reg = state.0.lock().await;
    Ok(reg.viewer_for_extension(&ext).map(PluginInfo::from))
}

/// Ask a column plugin to annotate a list of files. Returns one ColumnValue per file.
#[command]
pub async fn get_column_values(
    plugin_id: String,
    files: Vec<Value>,
    state: tauri::State<'_, PluginState>,
) -> Result<Vec<ColumnValue>, String> {
    let params = serde_json::json!({ "files": files });

    let result: Value = {
        let mut reg = state.0.lock().await;
        let proc = reg.get_or_spawn_process(&plugin_id)?;
        proc.call("get_columns", params).await?
    };

    let values = result["result"]
        .as_array()
        .ok_or("invalid result from plugin: expected array")?
        .iter()
        .filter_map(|v| {
            Some(ColumnValue {
                path: v["path"].as_str()?.to_string(),
                value: v["value"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect();

    Ok(values)
}

/// Invoke an action plugin on a set of files.
/// Returns a human-readable status message for the UI.
#[command]
pub async fn run_action_plugin(
    plugin_id: String,
    files: Vec<String>,
    target_dir: String,
    state: tauri::State<'_, PluginState>,
) -> Result<String, String> {
    let params = serde_json::json!({ "files": files, "target_dir": target_dir });

    let result: Value = {
        let mut reg = state.0.lock().await;
        let proc = reg.get_or_spawn_process(&plugin_id)?;
        proc.call("run_action", params).await?
    };

    Ok(result["result"]["message"]
        .as_str()
        .unwrap_or("Done")
        .to_string())
}

/// List a virtual directory via a panel plugin.
/// uri: e.g. "zip:///path/to/archive.zip/subdir"
#[command]
pub async fn panel_list_dir(
    plugin_id: String,
    uri: String,
    state: tauri::State<'_, PluginState>,
) -> Result<Vec<FileEntry>, String> {
    let params = serde_json::json!({ "uri": uri });

    let result: Value = {
        let mut reg = state.0.lock().await;
        let proc = reg.get_or_spawn_process(&plugin_id)?;
        proc.call("list_dir", params).await?
    };

    let entries = result["result"]["entries"]
        .as_array()
        .ok_or("invalid result from plugin: expected entries array")?
        .iter()
        .filter_map(|e| {
            let name = e["name"].as_str()?.to_string();
            let ext = std::path::Path::new(&name)
                .extension()
                .map(|x| x.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            Some(FileEntry {
                name: name.clone(),
                path: e["path"].as_str()?.to_string(),
                is_dir: e["is_dir"].as_bool().unwrap_or(false),
                is_symlink: false,
                size: e["size"].as_u64().unwrap_or(0),
                modified: e["modified"].as_u64().unwrap_or(0),
                extension: ext,
                is_hidden: name.starts_with('.'),
            })
        })
        .collect();

    Ok(entries)
}

/// Toggle a plugin's enabled state. Writes the updated flag back to plugin.json.
#[command]
pub async fn set_plugin_enabled(
    plugin_id: String,
    enabled: bool,
    state: tauri::State<'_, PluginState>,
) -> Result<(), String> {
    let mut reg = state.0.lock().await;

    let plugin_dir = reg
        .plugins
        .get(&plugin_id)
        .map(|p| p.plugin_dir.clone())
        .or_else(|| {
            // Plugin may be disabled (not in map) — look in plugins dir by id
            let dir = super::registry::PluginRegistry::plugins_dir().join(&plugin_id);
            if dir.is_dir() {
                Some(dir.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .ok_or("plugin not found")?;

    let manifest_path = std::path::PathBuf::from(&plugin_dir).join("plugin.json");
    let data = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let mut json: Value = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    json["enabled"] = Value::Bool(enabled);
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&json).unwrap(),
    )
    .map_err(|e| e.to_string())?;

    // Kill any running subprocess for this plugin if disabling
    if !enabled {
        reg.processes.remove(&plugin_id);
        reg.plugins.remove(&plugin_id);
    } else {
        // Re-load the manifest into the registry
        if let Some(loaded) =
            super::manifest::load_manifest(std::path::Path::new(&plugin_dir))
        {
            reg.plugins.insert(plugin_id.clone(), loaded);
        }
    }

    Ok(())
}

/// Install a plugin from a .nyx-plugin ZIP archive.
/// Extracts to ~/.nyx/plugins/<id>/ and chmod +x's the entrypoint for non-viewer plugins.
#[command]
pub fn install_plugin(zip_path: String) -> Result<(), String> {
    let plugins_base = super::registry::PluginRegistry::plugins_dir();
    std::fs::create_dir_all(&plugins_base).map_err(|e| e.to_string())?;

    let file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    // Validate: must contain plugin.json
    let manifest_json: Value = {
        let mut f = archive
            .by_name("plugin.json")
            .map_err(|_| "archive is missing plugin.json")?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(|e| e.to_string())?;
        serde_json::from_str(&s).map_err(|e| e.to_string())?
    };

    let id = manifest_json["id"]
        .as_str()
        .ok_or("plugin.json missing required 'id' field")?;
    let dest_dir = plugins_base.join(id);
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    // Extract all files
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = dest_dir.join(f.name());
        if f.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut out).map_err(|e| e.to_string())?;
        }
    }

    // chmod +x entrypoint for non-viewer plugins
    let category = manifest_json["category"].as_str().unwrap_or("viewer");
    if category != "viewer" {
        let entrypoint = manifest_json["entrypoint"].as_str().unwrap_or("plugin");
        let exe_path = dest_dir.join(entrypoint);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                &exe_path,
                std::fs::Permissions::from_mode(0o755),
            );
        }
    }

    Ok(())
}
