# Archive Browsing

Nyx.Commander can browse the following archive formats as if they were directories:

| Format | Extensions |
|--------|-----------|
| ZIP | `.zip` `.jar` `.war` `.ear` `.apk` `.ipa` `.docx` `.xlsx` `.pptx` `.odt` `.ods` |
| TAR + Gzip | `.tar.gz` `.tgz` |
| TAR + Bzip2 | `.tar.bz2` `.tbz2` |
| TAR + XZ | `.tar.xz` `.txz` |
| TAR (uncompressed) | `.tar` |

> **RAR / 7z:** These require native system tools. Place the cursor on such a file and press `Cmd+Enter` to open with the macOS default handler (e.g. The Unarchiver).

## Entering an Archive

Press **Enter** on any archive file. The panel switches into *archive mode*:

- A yellow **⟦arc⟧** bar replaces the path bar, showing `archive_name/current_inner_path`.
- The file list shows the archive's top-level contents (directories and files).
- Navigate into subdirectories with **Enter**, go up with **Backspace** or `..`.

## Navigating Inside an Archive

All the same navigation keys work inside archives:

| Key | Action |
|-----|--------|
| `Enter` on a dir | Descend into that directory |
| `Enter` on `..` | Go up one level inside the archive |
| `Backspace` | Go up one level inside the archive |
| `Enter` on `..` at archive root | **Exit the archive** and return to the filesystem |
| `×` button in the yellow bar | Exit the archive immediately |

> **Tip:** History navigation (`Alt+←` / `Alt+→`) tracks movements inside archives, so you can Back out of an archive without pressing `..` repeatedly.

## Extracting Files — F5

While inside an archive, **F5** extracts the selected files (or the cursor item) to the opposite panel's directory.

1. Select items inside the archive (Space / Ctrl+A) or place cursor on one.
2. Press **F5**.
3. Extraction starts immediately; no dialog. The opposite panel refreshes when done.

To extract the **entire archive**, exit the archive first (press `×` or `Backspace` at root), then copy the archive file itself via F5 — or use Finder / system tools.

## Reading Files Inside an Archive

- **Text files** (`.md`, `.rs`, `.json`, etc.) inside a ZIP can be viewed with the **Text Viewer** — press Enter on the file while inside the archive (extraction is done transparently).  
  *(Note: only implemented for ZIP; TAR formats require extraction first.)*
- **Cmd+Enter** on a file inside an archive extracts it to the opposite panel.

## Archive as Column-Based Navigation

Nyx.Commander treats archive VFS paths the same as filesystem paths in the history stack. You can:

- Open multiple tabs, one browsing an archive, another at the filesystem.
- Use Goto (`Cmd+G`) — the goto dialog only navigates the filesystem; use the archive's built-in navigation.
