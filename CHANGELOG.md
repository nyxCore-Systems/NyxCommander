# Changelog

All notable changes to Nyx.Commander are documented here.

---

## [1.2.0] — 2026-04-13

### Added
- **Column sorting** — click Name, Size, or Date headers to sort; click again to toggle direction; persists per-tab
- **Select by pattern** (`Ctrl+Shift+S`) — glob pattern input (`*.rs`, `test_*`, `img_???.png`)
- **Invert selection** (`Ctrl+Shift+I`)
- **Copy path to clipboard** (`Ctrl+Shift+C`)
- **Compress to ZIP** (`Ctrl+Shift+Z` or context menu) — select files → create `.zip` archive
- **Overwrite dialog** — copy/move now detect conflicts; choose Overwrite all, Skip existing, or Auto-rename
- **SHA-256 checksums** — compute and copy hashes for any file (context menu → Checksum)
- **Properties dialog** — full path, size, date, Unix permission display and editor (`chmod`)
- **Right-click context menu** — open, rename, copy, move, delete, copy path, compress, checksum, properties, open terminal, new folder
- **Open terminal here** (`Ctrl+Shift+T` or context menu) — opens Terminal at the active panel's path
- **Drive free space** in the status bar — shows free space for the current volume

### Backend (Rust)
- `check_conflicts` — pre-flight conflict check for copy/move
- `copy_items_mode` — copy with overwrite / skip / auto-rename strategy
- `create_zip` — ZIP archive creation from any file/directory selection
- `compute_checksum` — SHA-256 via `sha2` crate
- `open_terminal` — macOS: `open -a Terminal`; Linux: tries `x-terminal-emulator`, `gnome-terminal`, `konsole`, `xterm`
- `get_dir_size` — recursive directory byte count
- `get_file_mode` / `set_file_mode` — Unix permission bits (no-op on non-Unix)

---

## [1.1.0] — 2026-04-13

### Added
- **Plugin system** — four plugin categories: Viewer, Column, Action, Panel
- **Hot-reload** — plugins in `~/.nyx/plugins/` are picked up instantly without restart
- **Plugin Manager** — enable/disable, drag-and-drop `.nyx-plugin` install
- **Built-in plugins**: Image Viewer, Markdown Viewer, Git Status column
- **HelpViewer improvements** — full-text search with match highlighting, Left/Right focus split (sidebar ↔ content), link interception with Nyx-default / system-default toggle
- **Path bar click-to-edit** — clicking the path bar opens the Go To dialog; faint ✎ hint on hover
- **Mirror panel** (`Ctrl+=`) — open the active panel's path in the other panel

### Documentation
- `docs/user/plugins/index.md` — end-user plugin guide
- `docs/user/plugins/authoring.md` — plugin developer guide (viewer postMessage protocol, column/action/panel JSON-RPC, templates)

---

## [1.0.0] — 2026-04-12

Initial release.

### Core
- Dual-panel layout with independent tab stacks and navigation history per tab
- Archive browsing — walk into ZIP, TAR.GZ, TAR.BZ2, TAR.XZ, DOCX, XLSX and more with Enter/Backspace
- Quick filter — type in any panel to narrow the file list instantly
- Show hidden files toggle, flat recursive view

### File operations
- Copy (`F5`), Move (`F6`), Delete (`F8`), Rename (`F2`), New directory (`F7`)
- Multi-rename with pattern variables and live preview (`Cmd+M`)
- Selection: Space, Cmd+A, clear

### Viewers
- Text viewer — syntax-aware, 25+ languages, Markdown preview toggle
- Hex viewer — hex dump with ASCII pane and byte inspector
- Side-by-side diff viewer for text and images

### Tools
- Process explorer — live CPU + memory, kill any process (`F3`)
- Quick View pane — preview file under cursor without opening (`Cmd+Q`)
- Go To Path — fuzzy path input with history autocomplete (`Cmd+G`)
- Command bar — run shell commands in panel's directory (`` Ctrl+` ``)
- Directory sync — compare two folders, copy differences one-click (`d`)

### Customisation
- Four built-in themes: Neon, Classic, Matrix, Amber (`F9` to cycle)
- Custom theme editor
- Color rules — assign highlight colors by glob pattern or extension
- Favorites, Remotes bookmarks

### Platform
- macOS 12+ and Linux
- Native Rust backend (Tauri 2), SvelteKit frontend
