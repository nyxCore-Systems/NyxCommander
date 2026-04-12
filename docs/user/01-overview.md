# Overview

Nyx.Commander is a dual-pane file manager in the style of Norton Commander and Midnight Commander, built for macOS with a sci-fi pixel-art aesthetic. It is keyboard-driven but fully mouse-usable.

## Window Layout

```
┌────────────────────────────────────────────────────────────────┐
│ [Tab 1] [Tab 2] +  │  [Tab 1] +                               │
│─────────────────────│──────────────────────────────────────────│
│ /Users/you          │ /Users/you/Projects                       │
│──── Name ──── Size ─│─ Date ─ ║ ──── Name ──── Size ─── Date  │
│ 📁 ..               │ 📁 ..                                    │
│ 📁 Documents        │ 📁 nyx-commander                         │
│ 📁 Downloads        │ 📄 README.md                14K  today   │
│ 📄 notes.txt     2K │                                          │
│─────────────────────│──────────────────────────────────────────│
│ ‹ › 4 files, 3 dirs │ ‹ › 1 file, 1 dir                        │
│────────────────────────────────────────────────────────────────│
│ 1 Help  2 Rename  3 Procs  4 Open  5 Copy  6 Move  7 Mkdir    │
│         8 Delete  9 Theme  10 Quit  $_  QV                     │
└────────────────────────────────────────────────────────────────┘
```

- **Left and right panels** — each shows the contents of a directory.  
- **Tab bar** — each panel supports multiple tabs.  
- **Status bar** — file/dir counts; `‹ ›` history buttons.  
- **Function bar** — clickable shortcuts for common operations.  
- **Command bar** (`$_`) — inline shell, toggled with Ctrl+\`.  
- **Quick View** (`QV`) — side pane that previews the cursor file.

## Active Panel

Only one panel is *active* at a time (highlighted border). The active panel receives all keyboard input. Press **Tab** to switch panels. Click anywhere in a panel to make it active.

## Color Scheme

| Color | Meaning |
|-------|---------|
| Cyan  | Active panel border, accents, selected item outline |
| Green | Selected files |
| Yellow | Archive virtual directories |
| Red   | Dangerous actions (delete) |
| Dim   | Inactive panel, metadata |

The color scheme is fully themeable — see **[Customization](06-customization.md)**.
