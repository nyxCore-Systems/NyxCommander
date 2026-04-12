<div align="center">

<img src="src-tauri/icons/128x128@2x.png" width="108" alt="Nyx.Commander">

# Nyx.Commander

**Keyboard-first dual-pane file manager for macOS.**

[![Version](https://img.shields.io/badge/version-1.2.0-blue?style=flat-square)](src-tauri/tauri.conf.json)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey?style=flat-square)](https://tauri.app)
[![Built with Tauri](https://img.shields.io/badge/Tauri-2-24C8D8?style=flat-square&logo=tauri)](https://tauri.app)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?style=flat-square&logo=svelte)](https://svelte.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org)
[![Rust](https://img.shields.io/badge/Rust-CE422B?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)

</div>

---

Nyx.Commander is a native file manager for macOS and Linux in the tradition of Total Commander and FAR Manager — two panels, instant keyboard navigation, F-key file operations — wrapped in a crisp dark UI with pixel-art icons. The whole thing runs on a Rust backend via Tauri 2, with a SvelteKit frontend. No Electron, no web server.

What makes it different from most modern takes on the dual-pane formula: a **full plugin system** that lets you drop new viewers, columns, and virtual filesystems into `~/.nyx/plugins/` and have them appear live, no restart required.

<div align="center">

![Main view](images/Screenshot%202026-04-12%20at%2022.56.39.png)

</div>

---

## Features

**Navigation**
- Two independent panels, each with their own tab stack and navigation history
- Click the path bar to jump to any location, or use `Cmd+G` for fuzzy path input with autocomplete
- `Cmd+=` mirrors the active panel's path to the other — handy for copy/move workflows
- Quick filter: start typing in any panel to narrow the file list instantly

**Archive browsing** — walk into ZIP, TAR.GZ, TAR.BZ2, TAR.XZ, DOCX, XLSX and more with `Enter`, navigate subdirectories inside them, and `Backspace` back out. No extraction dialog.

**Viewers**
- Text viewer with monospace rendering for code and config files
- Hex viewer with ASCII pane, offset display, and byte inspector
- Side-by-side diff viewer for any two files

**Tools**
- Process explorer — live CPU + memory table, sortable, kill any process (`F3`)
- Quick View pane — sidebar preview of the file under the cursor (`Cmd+Q`)
- Command bar — run shell commands with the panel's path as working directory (`` Ctrl+` ``)
- Directory sync — compare two folders and copy differences in one click (`d`)
- Multi-rename — batch rename with pattern substitution (`Cmd+M`)

**Themes & color rules** — four built-in themes (Neon, Classic, Matrix, Amber), a custom theme editor, and per-extension or glob-pattern file colorization, all in the menu.

---

## Plugin System

Four plugin categories, loaded from `~/.nyx/plugins/` and hot-reloaded on change:

| Category | How it works | Example |
|----------|-------------|---------|
| **Viewer** | Self-contained HTML file rendered in a sandboxed iframe | Image viewer, Markdown renderer |
| **Column** | Any executable speaking JSON-RPC over stdin/stdout | Git status, image dimensions |
| **Action** | Same protocol, triggered by a keybinding | Compress selection, upload to S3 |
| **Panel** | Same protocol, mounts a virtual filesystem | S3 browser, FTP, ZIP browser |

Install a plugin by dropping a `.nyx-plugin` file onto the Plugin Manager. Three plugins ship built-in:

- **Image Viewer** — PNG, JPG, GIF, WebP, SVG, BMP, ICO; pixel-perfect scaling for small assets
- **Markdown Viewer** — renders `.md` / `.mdx` with headings, tables, and fenced code blocks
- **Git Status column** — adds a `Git` column showing working-tree status per file *(disabled by default, requires Python 3 + git)*

→ **[User guide: Plugins](docs/user/plugins/index.md)** · **[Plugin authoring guide](docs/user/plugins/authoring.md)**

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
<img src="images/Screenshot%202026-04-12%20at%2023.12.00.png" alt="Quick View pane showing an image">

</td>
<td align="center" width="50%">

**Go To Path**<br>
<img src="images/Screenshot%202026-04-12%20at%2022.58.02.png" alt="Go To Path dialog with history autocomplete">

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
<img src="images/Screenshot%202026-04-12%20at%2023.26.17.png" alt="Side-by-side Diff Viewer">

</td>
<td align="center" width="50%">

**Built-in Help** (`F1`)<br>
<img src="images/Screenshot%202026-04-12%20at%2022.57.13.png" alt="Help Viewer with full-text search">

</td>
</tr>
</table>

---

## Documentation

| | |
|---|---|
| [Overview](docs/user/01-overview.md) | What Nyx.Commander is and how it's structured |
| [Navigation](docs/user/02-navigation.md) | Panels, tabs, path bar, history, quick filter |
| [File Operations](docs/user/03-file-ops.md) | Copy, move, rename, delete, multi-rename |
| [Archives](docs/user/04-archives.md) | Browsing and extracting archive files |
| [Viewers](docs/user/05-viewers.md) | Text, hex, diff, and plugin viewers |
| [Tools](docs/user/06-tools.md) | Process explorer, quick view, command bar, dir sync |
| [Customization](docs/user/07-customization.md) | Themes, color rules, favorites, remotes |
| [Keyboard Shortcuts](docs/user/08-shortcuts.md) | Full shortcut reference |
| [Plugins — User Guide](docs/user/plugins/index.md) | Installing and managing plugins |
| [Plugins — Authoring Guide](docs/user/plugins/authoring.md) | Writing your own plugins |

Everything is also accessible from inside the app with `F1`.

---

## Keyboard Reference

| Key | Action |
|-----|--------|
| `Tab` | Switch active panel |
| `↑` `↓` `PgUp` `PgDn` | Move cursor |
| `Enter` | Enter dir / open file / browse archive |
| `Cmd+Enter` | Open with system app |
| `Backspace` | Go to parent |
| `Alt+←` / `Alt+→` | History back / forward |
| `Cmd+G` | Go to path |
| `Cmd+=` | Mirror current folder to other panel |
| `Space` | Toggle selection |
| `Cmd+A` | Select all |
| `Cmd+T` / `Cmd+W` | New / close tab |
| `Ctrl+1`–`9` | Switch to tab N |
| `Cmd+Q` | Toggle Quick View pane |
| `` Ctrl+` `` | Toggle command bar |
| `F1` | Help |
| `F2` | Rename |
| `F3` | Process explorer |
| `F4` | Open with system app |
| `F5` | Copy to other panel |
| `F6` | Move to other panel |
| `F7` | New directory |
| `F8` | Delete |
| `F9` | Cycle theme |
| `F10` | Quit |
| `d` | Directory sync (compare two panels) |
| `Shift+D` | Diff the two files under cursor (one per panel) |
| `Cmd+F` | Find files |
| `Cmd+H` | Hex viewer |
| `Cmd+M` | Multi-rename selected files |
| `Escape` | Open menu |

Type any printable character in a panel to start a quick filter.

---

## Building from Source

**Requirements:** macOS 12+ or Linux, [Rust](https://rustup.rs) stable, [Node.js](https://nodejs.org) 18+, [pnpm](https://pnpm.io)

```bash
pnpm install        # install JS dependencies
pnpm tauri dev      # dev server with hot-reload
pnpm check          # TypeScript + Svelte type check
pnpm tauri build    # production .app bundle
```

Built-in plugins live in `plugins/` and are copied to `~/.nyx/plugins/` on first launch. To work on a plugin, edit directly in `~/.nyx/plugins/<id>/` — the file watcher picks up changes within a second.

---

## License

MIT
