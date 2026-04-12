# Tools

## Find — Cmd+F

Full-text filename search starting from the active panel's current directory.

**Open:** `Cmd+F` (or `Ctrl+F`)

**Interface:**

- **Search box** — type the pattern (case-insensitive substring match).
- **Recursive toggle** — check to search the entire subtree; uncheck for the current directory only.
- **Results list** — shows matching files and directories with their type icon.

**Actions on results:**

- Click or navigate with `↑`/`↓` then press **Enter** — navigate the active panel to the directory containing that file.
- Press **Esc** — close the dialog.

Results are capped at 500 items. Dotfiles and hidden items are excluded from recursive search.

---

## Goto — Cmd+G

Type or paste any absolute path and navigate the active panel there directly.

**Open:** `Cmd+G` (or `Ctrl+G`)

**Features:**

- The input is pre-filled with the home directory (`~`).
- Press **Tab** to trigger path auto-completion — a dropdown shows matches from the parent directory.
- Use `↑`/`↓` to pick from the completion list, then **Tab** or **Enter** to accept.
- `~` is expanded to your home directory.
- **Enter** navigates to the typed path.
- **Esc** closes without navigating.

---

## Quick View — Ctrl+Q

A preview pane attached to the right side of the window that shows the content of the file under the cursor in the active panel.

**Toggle:** `Ctrl+Q` or click the **QV** button in the function bar.

**Preview types:**

| File type | Preview shown |
|-----------|---------------|
| Text / code | Full text content, scrollable |
| Image (png, jpg, gif, svg, webp, …) | Scaled image |
| Directory | Folder icon |
| Binary / large | Filename, extension, size |

The preview updates automatically as you move the cursor. It reads up to 256 KB for text previews.

---

## Command Bar — Ctrl+\`

An embedded shell prompt at the bottom of the window.

**Toggle:** `` Ctrl+` `` or click the **$\_** button in the function bar.

**Usage:**

- Type any shell command and press **Enter** to run it in `zsh`.
- The current working directory is the active panel's path (shown in the prompt).
- Command output (stdout + stderr) appears in the output area above the input.
- Press `↑`/`↓` to browse command history (last 50 commands).
- Press **Esc** to hide the command bar.

The command bar does not have a PTY — it is suitable for non-interactive commands (`ls`, `git status`, `brew install`, etc.). For interactive programs, use a real terminal.

> **Note:** The command bar does NOT change the panel's directory automatically. After running `cd`, the panel path is unchanged. Use the Goto dialog or navigate normally.

---

## Process Explorer — F3

An htop-style process list with system stats.

**Open:** **F3** or click **Procs** in the function bar.

**Display:**

- **System header:** CPU usage bar, memory usage bar, process count.
- **Process table:** PID, name, CPU %, memory (MB), status — sorted by CPU descending.

**Navigation:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor |
| `K` or `Delete` | Kill selected process (with confirmation) |
| `Esc` | Close |

The list auto-refreshes every 2 seconds while open.

> **Warning:** Killing a system or app process may cause data loss or instability. Confirm carefully.

---

## Directory Sync — D

Compare and synchronize two directories: left panel vs right panel.

**Open:** press **`D`** when no dialog is open.

**Interface:**

- **Top bar:** shows both directory paths being compared.
- **Filter toggle:** *Show identical* — check to also show files that are the same in both dirs.
- **Refresh button:** re-run the comparison.
- **Entry list:**

| Status | Meaning |
|--------|---------|
| `left only` | File exists only in the left directory |
| `right only` | File exists only in the right directory |
| `different` | File exists in both but sizes differ |
| `same` | File exists in both and sizes match |

**Actions on each row:**

- **`→`** button — copy from left to right.
- **`←`** button — copy from right to left.

After copying, the comparison is refreshed automatically.

---

## Background Transfer Queue

Copy and move operations can run in the background without blocking the UI.

**Access:** The **▶N** button (pulsing) in the function bar, where N = number of active/pending transfers.

**Queue dialog:**

- Lists all transfers with status icons: `◌` pending, `▶` running, `✓` done, `✕` error.
- Shows elapsed time and source/destination info.
- **Dismiss (×):** remove a single completed or errored job.
- **Clear done:** remove all completed jobs.

When a transfer errors, the error message is shown inline.
