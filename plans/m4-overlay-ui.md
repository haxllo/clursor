# M4: Overlay UI Rendering

## Goal
Full overlay UI: zoom loupe with pixel grid, color values (hex/RGB/HSL), color name, and swatch. All rendered via GDI into an always-on-top window.

## Rendering Approach

**Windows**: GDI rendering into a memory DC, then `BitBlt` to the window DC. Uses `StretchDIBits` for the loupe, `TextOutW` for labels, `FillRect` for swatches/background. No layered window or per-pixel alpha — solid dark navy background.

**History**: Removed from design. `Renderer` no longer tracks history.

## Overlay Layout (280×190)

```
┌──────────────────────────────────────┐
│ ┌──────────┐  #FF5733               │
│ │  6× zoom │  rgb(255,87,51)        │
│ │ 84×84    │  hsl(11,100%,60%)      │
│ │ 14×14 src│  Vermilion             │
│ └──────────┘                         │
│ ┌──────────────────────────────┐     │
│ │██████████████████████████████│     │  ← full-width swatch
│ └──────────────────────────────┘     │
│ ─────────────────────────────────    │  ← divider
│                                      │
└──────────────────────────────────────┘
```

## Implementation

### New: `renderer.rs`
- `Renderer` struct: holds capture buffer (no history)
- `paint()`: GDI rendering pipeline per frame
- Background: dark navy `FillRect` (no accent bar)
- Loupe: `StretchDIBits` for 14×14 → 84×84 nearest-neighbor (6× zoom)
- Grid: `CreatePen` + `MoveToEx/LineTo` at every 6th pixel (faint gray)
- Crosshair: gap-center design, `SetROP2(R2_NOT)` for pixel inversion
- Text: `CreateFontW` + `TextOutW` (22px bold hex, 15px body, 15px italic name)
- Swatch: `FillRect` with solid brush + `RoundRect` border with `NULL_BRUSH`
- Divider: horizontal line at y=134

### Changed: `overlay.rs`
- `apply_rounded_corners()` via `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE)` — smooth DWM anti-aliased corners

### Changed: `daemon.rs`
- Create `Renderer` instance (no history management)
- On each Active frame: update capture, request redraw
- `RedrawRequested`: call `renderer.paint()`

### Cargo.toml
- Added `Win32_UI_Controls` feature for `DwmSetWindowAttribute`

## Verification
1. `cargo run` → daemon starts
2. Hold Ctrl → overlay appears with dark background, loupe, text
3. Move cursor → loupe updates in real-time, hex/RGB/HSL/name change
4. Release Ctrl → overlay disappears, color copied to clipboard
5. Tray click → same behavior
