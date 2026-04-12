# Viewers

## Text Viewer

Opens text and source code files for read-only viewing inside the app.

**Triggered by:** pressing `Enter` on any recognized text file.

**Recognized file types:**

Plain text (`txt`, `md`, `rst`, `log`, `csv`), web (`html`, `css`, `scss`, `svg`), JavaScript / TypeScript (`js`, `jsx`, `ts`, `tsx`, `mjs`, `vue`, `svelte`, `astro`), systems languages (`rs`, `go`, `c`, `cpp`, `h`, `zig`, `nim`), scripting (`py`, `rb`, `lua`, `pl`, `sh`, `bash`, `zsh`, `fish`), data formats (`json`, `yaml`, `toml`, `xml`, `ini`, `env`), infrastructure (`tf`, `hcl`, `nix`, `dockerfile`, `makefile`), and more.

Dot-files with no extension (`.gitignore`, `.editorconfig`, etc.) are also treated as text.

**Keyboard shortcuts in the Text Viewer:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Scroll one line |
| `Page Up` / `Page Down` | Scroll one page |
| `Esc` | Close viewer |
| Button: *Open in editor* | Open with macOS default app |

The viewer shows line numbers on the left. Lines are soft-wrapped for wide content.

> **Size limit:** Files larger than 512 KB are not previewed (an error message is shown). Use the Hex Viewer for binary inspection of large files.

---

## Hex Viewer — Cmd+H

Opens any file in a classic hex dump view.

**Triggered by:** `Cmd+H` (or `Ctrl+H`) while the cursor is on a non-directory file.

**Layout:**

```
Offset    │ 00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F │ ASCII
──────────┼──────────────────────────────────────────────────┼──────────────────
00000000  │ 89 50 4E 47 0D 0A 1A 0A  00 00 00 0D 49 48 44 52 │ .PNG........IHDR
```

- **Offset column:** byte offset in hex.
- **Hex column:** 16 bytes per row, split into two groups of 8.
- **ASCII column:** printable bytes shown as characters; others as `.`

**Byte colour coding:**

| Color | Meaning |
|-------|---------|
| Dim / transparent | Null bytes (`0x00`) |
| Purple | Control characters (`0x01`–`0x1F`, `0x7F`) |
| Normal | Printable ASCII (`0x20`–`0x7E`) |
| Amber | High bytes (`0x80`–`0xFF`) |

**Status bar:** shows the byte under the cursor as: hex, decimal, octal, binary, and ASCII character.

**Keyboard shortcuts:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor one row (16 bytes) |
| `Page Up` / `Page Down` | Move one page (~38 rows) |
| `Esc` | Close viewer |

Files are loaded in 64 KB pages on demand — large files are handled efficiently.

---

## Diff Viewer — Shift+D

Compare the file under the cursor in the **left panel** with the file under the cursor in the **right panel**, side-by-side in unified diff format.

**Triggered by:** pressing `Shift+D` when both panels have a file under their respective cursors.

**Layout:**

```
─ /Users/me/a.txt   +++ /Users/me/b.txt            +3 added / −2 removed
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    │@@ -1,5 +1,6 @@
  1 │ unchanged line
  2 │ unchanged line
 -3 │ removed line
 +  │ added line A
 +  │ added line B
  4 │ unchanged line
```

**Row colours:**

| Color | Meaning |
|-------|---------|
| Green | Added line (`+`) |
| Red | Removed line (`−`) |
| Cyan | Hunk header (`@@`) |
| Dim | Unchanged context line |

The stats bar at the top shows total added / removed counts.

Special cases:
- **Binary files differ** — shown as a banner (diff does not produce line-level output).
- **Files are identical** — shown as a banner (no diff rows).

**Keyboard shortcuts:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Scroll |
| `Esc` | Close viewer |

---

## Directory Sync View — D

Compare the contents of the **left panel's directory** with the **right panel's directory** and selectively copy differences.

**Triggered by:** pressing `D` (without Shift) when no dialog is open.

See **[Tools → Directory Sync](05-viewers.md#directory-sync-view)** for full details — this is a combined view/action tool covered in the Tools chapter.
