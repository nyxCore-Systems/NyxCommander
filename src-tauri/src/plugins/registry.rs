use super::manifest::{load_manifest, LoadedPlugin, PluginCategory};
use super::subprocess::PluginProcess;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct PluginRegistry {
    /// All enabled discovered plugins, keyed by id.
    pub plugins: HashMap<String, LoadedPlugin>,
    /// Running subprocesses for column/action/panel plugins, keyed by plugin id.
    pub processes: HashMap<String, PluginProcess>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            processes: HashMap::new(),
        }
    }

    /// Returns the path to the user's plugin directory: ~/.nyx/plugins/
    pub fn plugins_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        PathBuf::from(home).join(".nyx").join("plugins")
    }

    /// Scan ~/.nyx/plugins/ and load all valid, enabled manifests.
    pub fn scan_plugins_dir(&mut self) {
        self.plugins.clear();

        let plugins_dir = Self::plugins_dir();
        let Ok(rd) = std::fs::read_dir(&plugins_dir) else {
            return;
        };

        for entry in rd.flatten() {
            if entry.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                if let Some(loaded) = load_manifest(&entry.path()) {
                    if loaded.manifest.enabled {
                        self.plugins.insert(loaded.manifest.id.clone(), loaded);
                    }
                }
            }
        }
    }

    /// Returns all enabled plugins of a given category.
    pub fn plugins_by_category(&self, cat: &PluginCategory) -> Vec<&LoadedPlugin> {
        self.plugins
            .values()
            .filter(|p| &p.manifest.category == cat)
            .collect()
    }

    /// Find the first viewer plugin that handles the given file extension.
    pub fn viewer_for_extension(&self, ext: &str) -> Option<&LoadedPlugin> {
        self.plugins_by_category(&PluginCategory::Viewer)
            .into_iter()
            .find(|p| {
                p.manifest
                    .file_extensions
                    .iter()
                    .any(|e| e.to_lowercase() == ext.to_lowercase())
            })
    }

    /// Get or spawn a subprocess for a plugin. Respawns if the process has died.
    pub fn get_or_spawn_process(
        &mut self,
        plugin_id: &str,
    ) -> Result<&mut PluginProcess, String> {
        // Remove dead process so we respawn below
        if let Some(proc) = self.processes.get_mut(plugin_id) {
            if !proc.is_alive() {
                self.processes.remove(plugin_id);
            }
        }

        if !self.processes.contains_key(plugin_id) {
            let exe = self
                .plugins
                .get(plugin_id)
                .ok_or_else(|| format!("plugin '{}' not found", plugin_id))?
                .entrypoint_path
                .clone();

            let proc =
                PluginProcess::spawn(&exe).map_err(|e| format!("spawn failed: {e}"))?;
            self.processes.insert(plugin_id.to_string(), proc);
        }

        Ok(self.processes.get_mut(plugin_id).unwrap())
    }
}

/// Tauri managed state wrapper.
/// Uses tokio::sync::Mutex so async commands can hold the guard across await points.
pub struct PluginState(pub tokio::sync::Mutex<PluginRegistry>);
