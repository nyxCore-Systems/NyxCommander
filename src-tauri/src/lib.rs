pub mod commands;
pub mod plugins;
use commands::*;
use plugins::commands::*;
use plugins::registry::{PluginRegistry, PluginState};
use std::sync::Mutex;
use sysinfo::System;
use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SystemState(Mutex::new(System::new())))
        .manage(PluginState(TokioMutex::new(PluginRegistry::new())))
        .setup(|app| {
            // Scan ~/.nyx/plugins/ at startup
            {
                let state = app.state::<PluginState>();
                let mut reg = state.0.blocking_lock();
                // Bootstrap built-in plugins on first run
                bootstrap_builtin_plugins();
                reg.scan_plugins_dir();
            }
            // Start filesystem watcher for hot-reload
            plugins::watcher::start_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ── Filesystem ──────────────────────────────────────────────────
            list_dir,
            list_volumes,
            copy_items,
            move_items,
            delete_items,
            create_dir,
            rename_path,
            open_file,
            get_home,
            read_file_bytes,
            read_text_file,
            diff_files,
            search_files,
            complete_path,
            // ── Processes ───────────────────────────────────────────────────
            list_processes,
            kill_process,
            get_system_stats,
            // ── Archives ────────────────────────────────────────────────────
            list_archive_dir,
            is_archive,
            extract_from_archive,
            // ── Misc ────────────────────────────────────────────────────────
            list_dir_flat,
            exec_shell,
            batch_rename,
            sync_dirs_list,
            // ── Plugin system ───────────────────────────────────────────────
            list_plugins,
            get_viewer_for_file,
            get_column_values,
            run_action_plugin,
            panel_list_dir,
            set_plugin_enabled,
            install_plugin,
            open_terminal,
            check_conflicts,
            copy_items_mode,
            create_zip,
            compute_checksum,
            get_dir_size,
            get_file_mode,
            set_file_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nyx.Commander");
}

/// Copy bundled built-in plugins to ~/.nyx/plugins/ on first run (if absent).
fn bootstrap_builtin_plugins() {
    let plugins_dir = PluginRegistry::plugins_dir();
    let _ = std::fs::create_dir_all(&plugins_dir);

    // Built-in plugin ids and their source paths (relative to the binary's directory).
    // At runtime, the `plugins/` directory is expected next to the binary or in the
    // project root for dev mode. We use CARGO_MANIFEST_DIR at compile time for dev.
    let builtin_src = {
        // In dev: use the project root plugins/ directory
        #[cfg(debug_assertions)]
        {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("plugins")
        }
        // In release: plugins/ is bundled alongside the binary
        #[cfg(not(debug_assertions))]
        {
            std::env::current_exe()
                .unwrap_or_default()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("plugins")
        }
    };

    if !builtin_src.is_dir() {
        return;
    }

    let Ok(entries) = std::fs::read_dir(&builtin_src) else {
        return;
    };

    for entry in entries.flatten() {
        if !entry.metadata().map(|m| m.is_dir()).unwrap_or(false) {
            continue;
        }
        let dest = plugins_dir.join(entry.file_name());
        if dest.exists() {
            continue; // Don't overwrite user-modified copies
        }
        copy_dir_recursive(&entry.path(), &dest);
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) {
    let _ = std::fs::create_dir_all(dst);
    let Ok(entries) = std::fs::read_dir(src) else {
        return;
    };
    for entry in entries.flatten() {
        let dst_path = dst.join(entry.file_name());
        if entry.metadata().map(|m| m.is_dir()).unwrap_or(false) {
            copy_dir_recursive(&entry.path(), &dst_path);
        } else {
            let _ = std::fs::copy(entry.path(), &dst_path);
            // Preserve executable bit
            #[cfg(unix)]
            if let Ok(meta) = entry.metadata() {
                use std::os::unix::fs::PermissionsExt;
                let mode = meta.permissions().mode();
                if mode & 0o111 != 0 {
                    let _ = std::fs::set_permissions(
                        &dst_path,
                        std::fs::Permissions::from_mode(mode),
                    );
                }
            }
        }
    }
}
