# tidewm

A minimal tiling window manager for Windows 10 and 11, written in Rust.
Zero configuration required. It runs, it tiles, it animates.

---

## What it does

tidewm sits in the background and automatically arranges every resizable
application window into a non-overlapping tiled layout. Open a new app —
it tiles. Close one — everything redistributes. No clicks, no dragging,
no thought required.

Windows that are not resizable (dialogs, small tools, fixed-size apps)
are left completely untouched. tidewm only manages windows that carry the
WS_SIZEBOX style flag, which is the Win32 signal for "this window can be
resized by the user."

---

## Why it exists

Windows has Snap, FancyZones, PowerToys — all fine, but they require
manual input every single time. tidewm is fully automatic. You open apps
and they arrange themselves. It is closer in spirit to i3, bspwm, or
Hyprland on Linux, but for Windows, written in Rust, and consuming almost
no system resources.

---

## Resource usage

- CPU at idle: ~0% (the main loop blocks on PeekMessageW + 10ms sleep)
- RAM: 2–4 MB
- The animation thread is spawned only during transitions and exits
  immediately when the animation completes

---

## Layouts

Four layouts are available, switchable at runtime via hotkeys:

```
TALL (default)              WIDE
+------------+-------+      +-------------------+
|            |   2   |      |        1          |
|     1      +-------+      +--------+----------+
|            |   3   |      |   2    |    3      |
+------------+-------+      +--------+----------+

BSP                         MONOCLE
+----------+-------+        +-------------------+
|    1     |   3   |        |                   |
+----------+-------+        |    1 (focused)    |
|    2     |   4   |        |                   |
+----------+-------+        +-------------------+
```

- **tall** — one main pane on the left, remaining windows stacked on the right.
- **wide** — one main pane on top, remaining windows stacked horizontally below.
- **bsp** — binary space partition, alternates horizontal and vertical splits.
- **monocle** — all windows occupy the full work area; only the focused one is visible.

---

## Animation

tidewm uses a cubic ease-out curve for all window transitions:

```
f(t) = 1 - (1 - t)^3
```

The result is a curve that decelerates smoothly to a clean stop with no
overshoot. The animation thread targets high frame rates using
`timeBeginPeriod(1)` for 1ms timer resolution. All windows in a single
frame are moved atomically using `DeferWindowPos` / `EndDeferWindowPos`
to avoid visual tearing between tiles.

We did try spring physics at some point. It looked terrible. lmfao.

---

## Windows invisible border compensation

Windows 10 and 11 add an invisible 7px border around every window for
the DWM drop shadow. Without compensation, placing two windows edge to
edge produces a visible gap. tidewm corrects for this in `apply_rect`:

```
actual_x = target_x - 7
actual_w = target_w + 14   (7px on each side)
actual_h = target_h + 7    (7px on the bottom only)
```

Windows appear flush with each other and with the screen edges.

---

## Hotkeys

The default modifier is Alt. Change it to Win or Ctrl in config.toml.

| Keys              | Action                          |
|-------------------|---------------------------------|
| Alt + H           | Focus previous window           |
| Alt + L           | Focus next window               |
| Alt + Shift + H   | Swap window left                |
| Alt + Shift + L   | Swap window right               |
| Alt + Space       | Toggle float for focused window |
| Alt + 1           | Switch to tall layout           |
| Alt + 2           | Switch to wide layout           |
| Alt + 3           | Switch to bsp layout            |
| Alt + 4           | Switch to monocle layout        |
| Alt + Enter       | Force re-tile all windows       |
| Alt + Q           | Quit tidewm                     |

---

## Configuration

tidewm reads `%APPDATA%\tidewm\config.toml` on startup.
If the file does not exist, it is created with defaults on first run.

```toml
# Gap in pixels between tiles (0 = flush edge to edge)
gap = 0

# Animation duration in milliseconds (0 = instant)
animation_ms = 160

# Layout: "tall" | "wide" | "bsp" | "monocle"
layout = "tall"

# Main pane ratio for tall and wide (0.0 to 1.0)
main_ratio = 0.55

# Hotkey modifier: "alt" | "win" | "ctrl"
modifier = "alt"
```

---

## Installation

### Option 1: installer (recommended)

Run `tidewm-setup.exe`. It will:

- Install `tidewm.exe` to `%LOCALAPPDATA%\tidewm\`
- Place `config.toml` in `%APPDATA%\tidewm\` (only if not already present)
- Create a Start Menu shortcut
- Register an auto-start entry in `HKCU\Run` (no admin required)
- Launch tidewm immediately

To uninstall, run `tidewm-setup.exe` again and choose **Uninstall**.
Your config.toml is preserved on uninstall.

### Option 2: manual

1. Copy `tidewm.exe` anywhere you want.
2. Run it.

To start with Windows automatically, place a shortcut to `tidewm.exe`
in `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`.

---

## Building from source

Requirements: Rust 1.70 or newer, Windows 10 SDK, MSVC or MinGW toolchain.

```powershell
cargo build --release
# Output: target\release\tidewm.exe
```

To build the installer:

```powershell
# First build tidewm itself
cargo build --release

# Then build the setup executable (from the setup\ directory)
cd setup
cargo build --release
# Output: setup\target\release\tidewm-setup.exe
```

The installer embeds `tidewm.exe` and `config.toml` at compile time via
`include_bytes!`. The resulting `tidewm-setup.exe` is fully self-contained.

The release profile uses opt-level 3, LTO, single codegen unit, and symbol
stripping. `tidewm.exe` is approximately 400–500 KB.

---

## Project structure

```
src/
  main.rs       entry point, loads config, starts WindowManager
  config.rs     reads %APPDATA%\tidewm\config.toml, typed defaults
  layout.rs     pure functions: screen rect + N windows -> N tile rects
  animator.rs   cubic ease-out evaluation, Animation and AnimationDriver
  manager.rs    core: window enumeration, event loop, hotkey handling
  hotkeys.rs    RegisterHotKey wrappers and hotkey ID constants

setup/
  src/main.rs   self-contained GUI installer (embeds tidewm.exe)
  Cargo.toml
  build.rs      embeds tidewm.ico into the setup executable
```

---

## Known limitations

- Single monitor only. Multi-monitor support is not implemented.
- Animation smoothness on Windows is limited by DWM compositor constraints.
  This is a platform limitation and cannot be resolved in userspace.
- Windows that resist being moved (some games, some fullscreen apps)
  will be detected but may not tile correctly.

---

## License

MIT. Do whatever you want with it.
