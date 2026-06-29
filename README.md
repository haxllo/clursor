# pick — Screen Color Picker

[![Rust](https://img.shields.io/badge/rust-2021-blue?logo=rust)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-windows%20|%20macOS%20|%20linux-lightgrey)](#)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](#)

**pick** is a screen color picker for designers, developers, and anyone who needs to grab colors from anywhere on their screen. Hold **Ctrl**, get an instant overlay with the pixel color under your cursor — hex, RGB, HSL, and a human-readable color name.

## Features

- **Ctrl hold-to-show** — press and hold Ctrl, overlay follows cursor in real time
- **Zoom loupe** — 6× magnification with pixel grid for pixel-perfect selection
- **Smart color naming** — CIEDE2000 perceptual distance against 949 XKCD color names (not basic X11)
- **Clipboard copy** — color copied automatically on Ctrl release
- **Tray click** — click tray icon to grab color without holding Ctrl
- **Configurable output** — hex (`#FF5733`), rgb (`rgb(255,87,51)`), or hsl (`hsl(11,100%,60%)`)
- **Cross-platform** — Windows (native), macOS and Linux (in progress)
- **Smooth overlay** — DWM-rounded corners, R2_NOT crosshair visible on any background

## Installation

### From source

```bash
# Prerequisites: Rust 1.81+
git clone https://github.com/YOUR_USER/pick.git
cd pick
cargo build --release
```

The binary is at `target/release/pick.exe` (Windows) or `target/release/pick`.

Add to your PATH or drop it somewhere convenient.

```bash
# Copy to a directory in your PATH
cp target/release/pick ~/.cargo/bin/
```

## Usage

### Daemon mode (default)

Start the system tray daemon:

```bash
pick
```

Hold **Ctrl** anywhere to activate the overlay. Release Ctrl to copy the color to your clipboard. Click the tray icon to grab a color at any time.

### Single capture

```bash
pick --once
```

Captures the pixel under your cursor, prints to stdout, and exits. Useful for scripting:

```bash
pick --once | tee >(xclip -selection clipboard)
```

## Configuration

On first run, pick creates `config.toml` in the default config directory:

| Platform | Path |
|----------|------|
| Windows  | `%USERPROFILE%\.config\pick\config.toml` |
| macOS    | `~/.config/pick/config.toml` |
| Linux    | `~/.config/pick/config.toml` |

```toml
default_format = "hex"       # "hex" | "rgb" | "hsl"
zoom_level = 4               # magnification (effective: 6× in current build)
history_size = 50            # (unused — history removed)
theme = "dark"               # "dark" | "light" | "system"
copy_on_click = true         # copy color when clicking overlay
show_color_name = true       # show name below color values
```

Open **Settings...** from the tray menu to edit the config file.

## How It Works

```
Screen capture (BitBlt, 96×96 around cursor)
        │
        ▼
  PixelAnalyzer samples center pixel
        │
        ├──► CIEDE2000 lookup → XKCD name
        └──► Format → hex / rgb / hsl
        │
        ▼
  GDI renderer paints overlay window
  (loupe, grid, crosshair, swatch, text)
        │
        ▼
  Ctrl release → clipboard copy
```

### Capture pipeline

The background thread polls `GetAsyncKeyState(VK_CONTROL)` at 250 Hz. When Ctrl is held:

1. `BitBlt` captures a 96×96 pixel region centered on the cursor
2. `GetDIBits` reads raw BGRA buffer
3. BGRA → RGBA conversion
4. Pixel at center (48,48) is the captured color
5. CIEDE2000 distance to 949 XKCD colors finds the closest name
6. Overlay rendered via GDI: loupe (14×14 → 84×84 at 6×), grid, crosshair, swatch, labels

### Overlay UI

```
┌──────────────────────────────────────┐
│ ┌──────────┐  #FF5733               │
│ │  6× zoom │  rgb(255,87,51)        │
│ │ 84×84    │  hsl(11,100%,60%)      │
│ │ 14×14 src│  Vermilion             │
│ └──────────┘                         │
│ ┌──────────────────────────────┐     │
│ │██████████████████████████████│     │
│ └──────────────────────────────┘     │
│ ─────────────────────────────────    │
│                                      │
└──────────────────────────────────────┘
```

280×190 window, 12px padding, DWM rounded corners, always-on-top, no focus steal.

## Key Bindings

| Key | Action |
|-----|--------|
| **Ctrl** (hold) | Show overlay, track color |
| **Ctrl** (release) | Hide overlay, copy color to clipboard |
| **Tray click** | Capture + copy instantly |

## Project Structure

```
src/
├── main.rs              # Entry point: --once or daemon mode
├── capture/
│   ├── mod.rs           # ScreenCapture trait + platform factory
│   ├── windows.rs       # BitBlt implementation (Win32 GDI)
│   ├── macos.rs         # CGDisplay stub (macOS)
│   └── linux.rs         # XShmGetImage stub (Linux)
├── color.rs             # Color struct, formatting, CIEDE2000 + XKCD lookup
├── config.rs            # TOML config load/save
├── daemon.rs            # System tray event loop, state machine, overlay lifecycle
├── overlay.rs           # Window positioning, platform styles, DWM corners
├── renderer.rs          # GDI rendering: loupe, grid, crosshair, swatch, text
├── hotkey.rs            # Platform::is_ctrl_held()
├── clipboard.rs         # arboard clipboard wrapper
├── xkcd_data.rs         # 949 XKCD color names (generated)
└── error.rs             # AppError, Result<T>
plans/
├── progress.md          # Milestone tracking
├── m1-daemon-core.md    # M1 spec
├── m2-system-tray-refinement.md
├── m3-overlay-window.md
└── m4-overlay-ui.md
```

## Development

```bash
# Build
cargo build

# Run in daemon mode
cargo run

# Run single capture
cargo run -- --once

# Release build
cargo build --release
```

### Prerequisites

- **Rust** 1.81+ (edition 2021)
- **Windows**: Windows SDK (GDI32, User32, DwmAPI)
- **macOS**: macOS 10.15+ (stubs only — not yet implemented)
- **Linux**: X11 with XShm extension (stubs only — not yet implemented)

## Color Name Quality

pick uses the [XKCD color name survey](https://blog.xkcd.com/2010/05/03/color-survey-results/) dataset (949 named colors) with CIEDE2000 perceptual distance, replacing Euclidean RGB distance used in simpler tools. This means color names are much closer to human perception — `#FF5733` is "Vermilion", not just "Red".

## Limitations

- **Windows focus**: macOS/Linux capture stubs exist but are untested
- **Single monitor edge flip**: overlay flips above cursor if near screen bottom
- **No DPI awareness**: overlay renders at logical coordinates (no per-monitor DPI scaling yet)

## License

MIT
