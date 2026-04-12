<div align="center">

<img src="src-tauri/icons/128x128@2x.png" width="96" alt="Nyx.Commander icon">

# Nyx.Commander

**A pixel-art dual-pane file manager for macOS — built with Tauri and Svelte.**

[![Version](https://img.shields.io/badge/version-1.0.0-blue?style=flat-square)](src-tauri/tauri.conf.json)
[![Platform](https://img.shields.io/badge/platform-macOS-lightgrey?style=flat-square&logo=apple)](https://tauri.app)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8D8?style=flat-square&logo=tauri)](https://tauri.app)
[![Svelte](https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&logo=svelte)](https://svelte.dev)
[![TypeScript](https://img.shields.io/badge/types-TypeScript-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org)
[![Rust](https://img.shields.io/badge/backend-Rust-CE422B?style=flat-square&logo=rust)](https://www.rust-lang.org)

</div>

---

<div align="center">

![Main view](images/Screenshot%202026-04-12%20at%2022.56.39.png)

</div>

## Overview

Nyx.Commander is a keyboard-first dual-pane file manager in the tradition of Total Commander and FAR Manager, reimagined as a native macOS app. It keeps the workflow that power users love — two panels, F-key operations, instant navigation — wrapped in a crisp dark UI with pixel-art icons and a full plugin system.

No Electron. No web server. Native Rust backend with a SvelteKit frontend rendered via Tauri's WebKit layer.

---

## Features

### Dual-panel navigation
Two independent panels, each with their own tab history. Switch with `Tab`, open a new tab with `Cmd+T`, jump to any path with `Cmd+G`, or click the path bar to edit it directly.

### Archive browsing
Browse ZIP, TAR, GZip, BZip2, XZ, DOCX, XLSX, and more as if they were directories — press `Enter` to walk in, `Backspace` to walk out. No extraction required.

### Viewers
- **Text viewer** — syntax-aware, monospace, scrollable (`Enter` on any text/code file)
- **Hex viewer** — full hex dump with ASCII pane and status bar
- **Diff viewer** — side-by-side colored diff between any two files
- **Image viewer** — via the built-in plugin; pixel-perfect scaling for small pixel art assets

### Tools
- **Process explorer** — live CPU + memory table, kill any process (`F3`)
- **Quick View pane** — preview the file under the cursor without opening a dialog (`Cmd+Q`)
- **Go To Path** — fuzzy path input with history autocomplete (`Cmd+G`)
- **Command bar** — run shell commands with the current path as `$cwd` (`` Ctrl+` ``)
- **Dir sync** — compare two directories side by side, copy differences in one click (`d`)
- **Multi-rename** — batch-rename selected files with pattern substitution (`Cmd+M`)

### Plugin system
Four plugin categories, hot-reloaded from `~/.nyx/plugins/` — no restarts needed:

| Category | What it does |
|----------|-------------|
| **Viewer** | Renders files in a fullscreen overlay (HTML iframe, sandboxed) |
| **Column** | Adds a metadata column to both panels (subprocess, any language) |
| **Action** | Keyboard-triggered operation on selected files (`Ctrl+Shift+<key>`) |
| **Panel** | Mounts a virtual filesystem (S3, FTP, custom protocols) |

Drag a `.nyx-plugin` file onto the Plugin Manager to install. Built-in: Image Viewer, Markdown Viewer, Git Status column.

### Themes & color rules
Four built-in themes — Neon, Classic, Matrix, Amber — switchable at any time with `F9`. Full custom theme editor in the menu. Assign highlight colors to files by glob pattern or extension.

---

## Screenshots

<table>
<tr>
<td align="center" width="50%">

**Plugin Manager**<br>
<img src="images/Screenshot%202026-04-12%20at%2022.56.56.png" alt="Plugin Manager">

</td>
<td align="center" width="50%">

**Theme Picker**<br>
<img src="images/Screenshot%202026-04-12%20at%2022.56.52.png" alt="Theme Picker">

</td>
</tr>
<tr>
<td align="center" width="50%">

**Quick View — Image Preview**<br>
<img src="images/Screenshot%202026-04-12%20at%2023.12.00.png" alt="Quick View with image preview">

</td>
<td align="center" width="50%">

**Go To Path**<br>
<img src="images/Screenshot%202026-04-12%20at%2022.58.02.png" alt="Go To Path dialog with autocomplete">

</td>
</tr>
<tr>
<td align="center" width="50%">

**Hex Viewer**<br>
<img src="images/Screenshot%202026-04-12%20at%2022.59.26.png" alt="Hex Viewer">

</td>
<td align="center" width="50%">

**Process Explorer**<br>
<img src="images/Screenshot%202026-04-12%20at%2022.59.08.png" alt="Process Explorer">

</td>
</tr>
<tr>
<td align="center" width="50%">

**Diff Viewer**<br>
<img src="images/Screenshot%202026-04-12%20at%2023.26.17.png" alt="Diff Viewer">

</td>
<td align="center" width="50%">

**Built-in Help**<br>
<img src="images/Screenshot%202026-04-12%20at%2022.57.13.png" alt="Help Viewer with search">

</td>
</tr>
</table>

---

## Keyboard Reference

| Key | Action |
|-----|--------|
| `Tab` | Switch active panel |
| `Enter` | Enter dir / open file / browse archive |
| `Cmd+Enter` | Open with system app |
| `Backspace` | Go to parent |
| `Alt+←` / `Alt+→` | History back / forward |
| `Cmd+G` | Go to path |
| `Cmd+=` | Mirror current folder to other panel |
| `F1` | Help |
| `F2` | Rename |
| `F5` | Copy to other panel |
| `F6` | Move to other panel |
| `F7` | New directory |
| `F8` | Delete |
| `F9` | Cycle theme |
| `Space` | Toggle selection |
| `Cmd+A` | Select all |
| `Cmd+T` / `Cmd+W` | New / close tab |
| `Cmd+Q` | Quick View pane |
| `` Ctrl+` `` | Command bar |
| `Escape` | Open menu |

Type any printable character in a panel to start a quick filter.

---

## Requirements

- macOS 12 Monterey or later
- [Rust](https://rustup.rs) (stable toolchain)
- [Node.js](https://nodejs.org) 18+ and [pnpm](https://pnpm.io)

---

## Development

```bash
# Install dependencies
pnpm install

# Start dev server (hot-reload)
pnpm tauri dev

# Type check
pnpm check

# Production build
pnpm tauri build
```

Built-in plugins live in `plugins/` at the repo root and are bootstrapped to `~/.nyx/plugins/` on first launch. To iterate on a plugin, edit in `~/.nyx/plugins/<id>/` directly — the watcher picks up changes within a second.

---

## Writing Plugins

See **[docs/user/plugins/authoring.md](docs/user/plugins/authoring.md)** for the full developer guide, or press `F1` inside the app and navigate to **Plugin Dev**.

Quick start — a Python column plugin:

```python
#!/usr/bin/env python3
import sys, json

for line in sys.stdin:
    req = json.loads(line.strip())
    if req['method'] == 'get_columns':
        results = [{'path': f['path'], 'value': '✓'} for f in req['params']['files']]
        print(json.dumps({'id': req['id'], 'result': results}), flush=True)
```

Drop it in `~/.nyx/plugins/com.you.myplugin/` with a `plugin.json` manifest. Done.

---

## License

MIT
