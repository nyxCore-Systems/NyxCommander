# Plugins

Nyx.Commander supports plugins that extend it with new file viewers, custom panel columns, keyboard-triggered actions, and virtual filesystems. Plugins are regular files dropped into a folder — no restarts, no app store.

---

## Plugin Types

| Type | What it does | Triggered by |
|------|-------------|-------------|
| **Viewer** | Renders a file type in a fullscreen overlay | `Enter` on a matching file |
| **Column** | Adds a metadata column to both panels | Automatic when the directory loads |
| **Action** | Runs a custom operation on selected files | `Ctrl+Shift+<key>` |
| **Panel** | Mounts a virtual filesystem as a panel source | Protocol prefix (e.g. `zip://`) |

---

## Plugin Directory

Plugins live in:

```
~/.nyx/plugins/<plugin-id>/
```

Each plugin is a folder containing a `plugin.json` manifest and its entrypoint file. Nyx watches this directory — adding, removing, or editing a `plugin.json` takes effect immediately without restarting the app.

---

## Plugin Manager

Open the Plugin Manager via **Menu → Plugins** (`Esc` to open the menu, then select **Plugins** in the sidebar).

The Plugin Manager shows all installed plugins grouped by type. For each plugin you can:

- **Toggle the checkbox** to enable or disable it. The change is written back to `plugin.json` instantly.
- **Drop a `.nyx-plugin` file** onto the install zone at the bottom to install a new plugin.
- Click **Open plugins folder** to open `~/.nyx/plugins/` in Finder.

---

## Installing Plugins

### Drag and drop

1. Obtain a `.nyx-plugin` file (a ZIP archive renamed with the `.nyx-plugin` extension).
2. Open the Plugin Manager.
3. Drop the file onto the **"Drop a .nyx-plugin file here to install"** zone.

The plugin is extracted to `~/.nyx/plugins/<id>/`, made executable if needed, and appears in the list automatically.

### Manual install

Copy or symlink a folder containing `plugin.json` into `~/.nyx/plugins/`. The watcher picks it up within a second.

```bash
cp -r ~/Downloads/my-plugin ~/.nyx/plugins/com.example.my-plugin
```

### Disabling without uninstalling

Uncheck the toggle in Plugin Manager, or edit `plugin.json` and set `"enabled": false`.

---

## Built-in Plugins

These ship with Nyx.Commander and are automatically copied to `~/.nyx/plugins/` the first time the app starts. Pre-existing copies are never overwritten.

### Image Viewer — `com.nyx.image-viewer`

Renders images inside Nyx without opening an external app.

**Supported formats:** `png` `jpg` `jpeg` `gif` `webp` `svg` `bmp` `ico`

**Triggered by:** pressing `Enter` on a matching image file.

**Status bar** shows dimensions and format, e.g. `1024×768 · PNG`.

Small images (≤ 128 × 128 px) are displayed with pixel-perfect scaling — intentional for pixel art assets.

### Markdown Viewer — `com.nyx.markdown-viewer`

Renders `.md`, `.mdx`, and `.markdown` files as formatted HTML, including:

- Headings, bold, italic, blockquotes
- Fenced code blocks
- Tables
- Links

**Status bar** shows line and word count.

> **Note:** The renderer (`marked.js`) is bundled locally inside the plugin — no network requests are made.

### Git Status Column — `com.nyx.git-status`

Adds a **Git** column to both panels showing the working-tree status of each file.

**Disabled by default.** Enable it in Plugin Manager.

**Requires:** Python 3 and `git` on your `PATH`.

| Value | Meaning |
|-------|---------|
| `✓` | Tracked, no changes |
| `~ modified` | Modified (`M`) |
| `+ added` | Staged new file (`A`) |
| `- deleted` | Deleted (`D`) |
| `→ renamed` | Renamed (`R`) |
| `! conflict` | Merge conflict (`U`) |
| `? untracked` | Not tracked by git |
| *(empty)* | Not inside a git repository |

The plugin calls `git status --porcelain` per file with a 3-second timeout. In non-git directories the column stays blank.

---

## Viewer Keyboard Shortcuts

When a viewer plugin is open:

| Key | Action |
|-----|--------|
| `Esc` | Close the viewer |

The viewer itself may support additional keys (scroll, zoom, etc.) depending on the plugin — check the plugin's own documentation.

---

## Uninstalling a Plugin

Remove the plugin's folder from `~/.nyx/plugins/`:

```bash
rm -rf ~/.nyx/plugins/com.example.my-plugin
```

The plugin disappears from the list immediately.
