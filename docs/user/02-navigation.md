# Navigation

## Moving the Cursor

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor one row |
| `Page Up` / `Page Down` | Move cursor 20 rows |
| `Enter` | Enter directory / open file |
| `Backspace` | Go to parent directory |
| `Tab` | Switch active panel |

## Entering Directories

Press **Enter** on any directory row (including `..` to go up). You can also **double-click** any row.

## Opening Files

| Situation | Key | Result |
|-----------|-----|--------|
| Text / code / config file | `Enter` | Open in built-in Text Viewer |
| Archive (zip, tar.gz, …) | `Enter` | Browse archive as a virtual directory |
| Any file | `Cmd+Enter` | Open with macOS default application |
| Any file | `F4` | Open with macOS default application |
| Binary / unknown | `Enter` | Open with macOS default application |

## Path Bar

The path bar at the top of each panel shows the current path as clickable breadcrumbs. Click any segment to navigate directly to that directory.

## Quick Filter

Start typing any printable character while the panel is active — a filter bar appears at the bottom:

- **Type** to narrow the list in real time.
- **Enter** moves cursor to the first match and closes the filter.
- **Esc** clears the filter and closes the bar.
- The match count shows as `N/M` on the right.

The filter is case-insensitive and matches anywhere in the filename. It is cleared automatically on navigation.

## Tabs

Each panel supports multiple tabs. Each tab has its own path, cursor, selection, and history.

| Key | Action |
|-----|--------|
| `Ctrl+T` | Open new tab (same path) |
| `Ctrl+W` | Close current tab |
| `Ctrl+1`–`9` | Jump to tab by number |
| Click tab | Switch to that tab |
| Click `×` | Close that tab |
| Click `+` | Open new tab |

Tabs are displayed above each panel. The active tab is highlighted with an accent-colored top border.

## History Navigation

Every navigation action is recorded in a per-tab history stack.

| Key | Action |
|-----|--------|
| `Alt+←` | Go back |
| `Alt+→` | Go forward |
| `‹` button (status bar) | Go back |
| `›` button (status bar) | Go forward |

History tracks both filesystem paths and positions inside archives.

## Flat View

Click the **≡** button in the column header (or toggle in the panel) to see *all files in the current directory tree* as a single flat list — useful for scanning large projects.

- Only files are shown (no directories in the list).
- Paths are the full filesystem paths.
- Navigate to a file's directory with Enter or F5/F8 as normal.
- Click **≡** again to exit flat view.

## Hidden Files

Click the **.** button in the column header to toggle visibility of hidden files (those starting with `.`). The setting is per-panel.

## Volumes / Drives

Use **Escape → Favorites → Volumes** (or navigate to `/Volumes`) to access mounted drives. The Goto dialog (`Cmd+G`) also accepts `/Volumes/DiskName` directly.
