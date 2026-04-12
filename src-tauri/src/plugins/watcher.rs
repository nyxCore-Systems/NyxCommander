use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use tauri::{AppHandle, Emitter, Manager};

use super::registry::{PluginRegistry, PluginState};

/// Start a filesystem watcher on ~/.nyx/plugins/.
/// On any change: debounce 500ms, rescan the registry, emit "plugins-changed" to the frontend.
pub fn start_watcher(app_handle: AppHandle) {
    let plugins_dir = PluginRegistry::plugins_dir();
    // Ensure the directory exists before watching
    let _ = std::fs::create_dir_all(&plugins_dir);

    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[nyx plugins] watcher init failed: {e}");
                return;
            }
        };

        if let Err(e) = watcher.watch(&plugins_dir, RecursiveMode::NonRecursive) {
            eprintln!("[nyx plugins] watch failed: {e}");
            return;
        }

        for res in rx {
            if res.is_ok() {
                // Debounce
                std::thread::sleep(std::time::Duration::from_millis(500));

                // Rescan
                if let Some(state) = app_handle.try_state::<PluginState>() {
                    state.0.blocking_lock().scan_plugins_dir();
                }

                // Notify frontend
                let _ = app_handle.emit("plugins-changed", ());
            }
        }
    });
}
