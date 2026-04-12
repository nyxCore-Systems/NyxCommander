# File Operations

## Selection

Before most operations, select the files you want to act on.

| Key | Action |
|-----|--------|
| `Space` | Toggle selection on cursor item, move cursor down |
| `Ctrl+A` / `Cmd+A` | Select all items in the panel |

Selected items are highlighted green. If nothing is selected, the operation acts on the item under the cursor.

## Copy — F5

Copies selected files/directories to the opposite panel's directory.

1. Select files (or place cursor on one file).
2. Press **F5**. A dialog opens with source list and pre-filled destination (the other panel's path).
3. Edit the destination if needed.
4. Press **Enter** or click **Copy**.

Directories are copied recursively. Existing files at the destination are overwritten.

> **Tip:** Copy runs synchronously in a dialog. For large transfers, consider the **Background Transfer Queue** — select files, use the Queue button instead of F5, then continue working.

## Move — F6

Works identically to Copy but removes the source after a successful copy. Tries an atomic rename first (instant for same-volume moves); falls back to copy+delete for cross-volume.

1. Select files.
2. Press **F6**.
3. Confirm destination and press **Enter**.

## Delete — F8

Permanently deletes selected files and directories. **No Trash — this is irreversible.**

1. Select files.
2. Press **F8**.
3. A confirmation dialog lists the items.
4. Press **Enter** or click **Delete** to confirm, **Esc** to cancel.

## New Directory — F7

Creates a new directory inside the active panel's current path.

1. Press **F7**.
2. Type the directory name (nested paths like `a/b/c` are created with `mkdir -p`).
3. Press **Enter**.

## Rename — F2

Renames the file or directory under the cursor.

1. Navigate cursor to the item.
2. Press **F2**.
3. Edit the name in the input field.
4. Press **Enter** to confirm, **Esc** to cancel.

## Multi-Rename — Ctrl+M

Batch-rename multiple files with a powerful pattern system.

1. **Select** the files to rename (Space or Ctrl+A).
2. Press **Ctrl+M** (or F2 when multiple items are selected).
3. Configure the rename pattern:

| Token | Replaced with |
|-------|--------------|
| `$F`  | Original full filename including extension |
| `$1`  | Original name without extension |
| `$E`  | Original extension (without the dot) |
| `$N`  | Counter (see counter settings) |

**Examples:**

| Pattern | Input | Counter | Output |
|---------|-------|---------|--------|
| `$1_backup.$E` | `photo.jpg` | — | `photo_backup.jpg` |
| `IMG_$N.$E` | `DSC001.jpg` | start=1 | `IMG_001.jpg` |
| `$N - $F` | `song.mp3` | start=1,pad=2 | `01 - song.mp3` |

**Search/Replace mode:** Check *Regex* and fill the *Search* field to do a find-replace on the full filename instead of using the token pattern.

**Counter settings:**

- *Start* — first counter value (default 1)
- *Step* — increment per file (default 1)
- *Pad width* — zero-pad to this many digits (default 2, so `01`, `02`, …)

A live preview shows every rename before you apply. Press **Cmd+Enter** or click **Rename** to apply.

## Open with System App — F4 / Cmd+Enter

Opens the cursor file using macOS's default application (same as double-clicking in Finder).

## Background Transfer Queue

The transfer queue lets copy/move operations run without blocking the UI.

- Transfers enqueue immediately and run in the background.
- The **▶N** button in the function bar pulses when transfers are active (N = count).
- Click **▶N** to open the queue dialog showing status of all jobs.
- Completed jobs can be dismissed individually or all at once.
