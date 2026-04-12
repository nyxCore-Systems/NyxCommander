# Customization

Open the main menu with **Escape**. Use `↑`/`↓` to navigate the sidebar, `Enter` to activate, `→`/`←` to move between sidebar and content.

---

## Themes — F9 / Escape → Themes

Press **F9** to cycle through built-in themes. Open the menu and navigate to **Themes** for more options and the custom theme editor.

### Built-in Themes

| Name | Description |
|------|-------------|
| **Nyx** | Default — deep navy background, cyan accents, green selection |
| **Abyss** | Dark charcoal with teal/magenta |
| **Phosphor** | Classic green-on-black terminal |
| **Ash** | Muted grey, low contrast |

### Custom Theme Editor

In the **Themes** section of the menu:

1. Click **New theme** (or navigate to it with keyboard and press Enter).
2. A live editor opens with all CSS variable groups:
   - **Backgrounds** — panel, header, function bar, row states
   - **Borders** — panel border, dim border, cursor/selection outlines
   - **Text** — main text, dim text, directory color, selection color, danger color
   - **State Indicators** — accents, scrollbar, etc.
3. Click any color swatch to open the color picker. Changes apply immediately (live preview).
4. Type a name for your theme.
5. Click **Save** — the theme is stored in `localStorage` and persists across sessions.

### Deleting Custom Themes

In the Themes content area, navigate to a custom theme and press **Delete** (or click the trash icon).

---

## Color Rules — Escape → Settings → Color Rules

Highlight file rows by extension, name pattern, or size. Rules are evaluated top-to-bottom; the first matching rule wins.

**Open:** Escape → Settings → Color Rules (or `uiStore.setDialog({ kind: 'color-rules' })` via dev tools).

### Default Rules

| Rule | Extensions / Pattern | Color |
|------|---------------------|-------|
| Archives | zip tar gz bz2 xz rar 7z jar war apk | Yellow |
| Media | mp4 mkv mp3 flac ogg wav | Purple |
| Images | png jpg jpeg gif svg webp | Cyan |
| Executables | sh bash Makefile | Green bold |
| Large files (>100 MB) | any | Red |

### Creating a Rule

1. Click **+ Add rule**.
2. Set the **label** (display name only).
3. Set one or more **match criteria** (OR logic — any matching criterion triggers the rule):
   - **Extensions** — comma-separated, e.g. `rs,toml,lock`
   - **Name contains** — comma-separated substrings, e.g. `Makefile,README`
   - **Min size** — minimum file size in bytes (0 = ignored)
4. Set the **style**:
   - **FG** — foreground/text color (color picker)
   - **BG** — background color (optional; click ✕ to clear)
   - **Bold** / **Italic** checkboxes
5. Use the **↑**/**↓** buttons to reorder rules.
6. The swatch on the right shows a live preview.

Rules are saved immediately to `localStorage`.

---

## Favorites — Escape → Favorites

Bookmarks for frequently-visited directories.

### Adding a Favorite

From the **Favorites** section of the menu:

- Click **+ Add current path** (or navigate to it and press Enter) to add the active panel's current path.
- A default name is suggested (the directory name); you can rename it after adding.

### Using Favorites

- Navigate to a favorite: press **Enter** on it (or double-click).
- **Rename:** press **F2** on a favorite in the list.
- **Delete:** press **Delete** (or `Backspace`) on a favorite.

Favorites are stored in `localStorage` and persist across sessions.

---

## Remotes — Escape → Remotes

Configure SSH, SFTP, FTP, and CIFS (SMB) connections. Nyx.Commander opens remote connections using the system's default protocol handler — your macOS default SFTP/SSH client (e.g. Finder for SMB, or a registered SSH handler).

### Adding a Remote

1. Navigate to **Remotes** in the menu.
2. Click **+ New remote** or press Enter on it.
3. Fill in the form:
   - **Name** — display name
   - **Type** — SSH / SFTP / FTP / CIFS
   - **Host** — hostname or IP
   - **Port** — default is filled automatically per type (22, 21, 445)
   - **User** — username (optional)
   - **Password** — stored locally in `localStorage` (not encrypted — use SSH keys for security)
   - **Key path** — path to SSH private key (SSH/SFTP only)
   - **Remote path** — initial path to open
4. Click **Save**.

### Connecting

Press **Enter** on a saved remote to connect. This builds a URL (`sftp://user@host/path`) and opens it with the macOS default handler. Finder handles `smb://`, your registered SSH app handles `ssh://`, etc.

### Editing / Deleting Remotes

- Navigate to a remote in the list and press **F2** to edit, **Delete** to remove.

---

## Settings — Escape → Settings

| Setting | Description |
|---------|-------------|
| Show hidden files | Toggle `.` files visibility (also via the `.` button in panel header) |
| Color Rules | Opens the color rules editor |

Settings are per-session (hidden files toggle) or global (color rules in localStorage).
