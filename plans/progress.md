# Progress — 2026-06-29

## Status

- **M1**: ✅ Complete — Daemon core + screen capture + system tray
- **M2**: ✅ Complete — System tray refinement + Ctrl trigger
- **M3**: ✅ Complete — Overlay window
- **M4**: ✅ Complete — Overlay UI rendering
- **M5**: ⬜ Not started — Input handling + lock mode + config
- **M6**: ⬜ Not started — Harmonies + advanced features
- **M7**: ⬜ Not started — Polish + distribution

---

## M1 Delivered

- `pick --once` captures pixel under cursor, prints `#HEX  Name`
- `pick` starts system tray daemon with purple circle icon
- Screen capture via BitBlt (Windows), stubs for macOS/Linux
- Config file at `~/.config/pick/config.toml` (auto-created on first run)
- Tray menu: Settings... | Quit

## M2 Delivered

- **Trigger**: Ctrl key (hold-to-show). ALT was rejected — it flashes menu bars/shifts layout in native apps like Chrome, stealing focus and changing the pixel under cursor.
- **State machine**: Idle ↔ Active transitions with tracing events
- **Ctrl release → clipboard copy**: last tracked color copied on release
- **Tray click → pick**: capture + copy without holding Ctrl (same as holding/release)
- **Tooltip updates**: tray icon shows last picked color in tooltip
- **Settings**: opens config file in default editor
- **Config integration**: `--once` respects `default_format` (hex/rgb/hsl)
- **Color naming**: CIEDE2000 perceptual distance + 949 XKCD color names
  - Euclidean RGB replaced with CIEDE2000 (human-eye-perceptual distance)
  - 130 X11 names replaced with 949 names from XKCD color name survey
- **Crate versions**: all latest stable as of June 2026

## M3 Delivered

- **Overlay window**: winit window, no decorations, always-on-top, hidden by default
- **Platform styles**: `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`
- **Visibility toggle**: show on Ctrl press, hide on Ctrl release
- **Cursor tracking**: reposition to (cursor + 12px offset) on each frame
- **Screen edge flip**: if cursor near bottom, position above instead
- **Rounded corners**: `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND)` — DWM smooth anti-aliased corners (not `SetWindowRgn` which produces jagged aliased edges)

## M4 Delivered

- **Full GDI rendering pipeline**: background, zoom loupe, pixel grid, crosshair, color swatch, text labels
- **Loupe**: 14×14 source pixels from capture center, zoomed 6× to 84×84 via `StretchDIBits`
- **Pixel grid**: 15×15 faint lines at 6px spacing, fixed color `gdi_color(50, 50, 65)`
- **Crosshair**: gap-center design — 4 short arms separated by 4px center gap. Uses `SetROP2(R2_NOT)` to invert pixels beneath for visibility on any color. 40px total drawn (vs ~100px with full through-lines)
- **Color values**: hex (22px bold white), RGB (15px gray), HSL (15px gray), name (15px italic purple accent)
- **Full-width swatch**: 256×20px with rounded border
- **Divider**: separator line
- **No accent bar**: removed from design
- **No loupe border**: removed from design
- **History removed**: feature eliminated, `Renderer` no longer tracks history
- **Layout**: 280×190 window, 12px padding

---

## Pixel Detection — How It Works

### What pixel is captured

The tool captures the **pixel at the exact center** of a 96×96 region around the cursor.

```
The cursor tip is at (x, y) from GetCursorPos.

BitBlt grabs region: (x-48, y-48) to (x+48, y+48)  → 96×96 pixels

PixelAnalyzer::sample_center reads the center pixel:
    cx = 96 / 2 = 48
    cy = 96 / 2 = 48
    index = (48 * 96 + 48) * 4  → offset into RGBA buffer
    → returns (r, g, b)
```

So the color shown is always the **single pixel directly under the cursor tip**.

### BitBlt capture pipeline (Windows)

```
1. GetDC(NULL)          → screen device context (entire virtual screen)
2. CreateCompatibleDC()  → offscreen memory DC
3. CreateCompatibleBitmap(96×96)  → DDB surface
4. SelectObject(bitmap into memory DC)
5. BitBlt(96×96 from screen DC → memory DC)  → copies pixels
6. GetDIBits(bitmap → raw BGRA buffer)
7. BGRA → RGBA conversion (swap bytes 0 and 2)
8. Cleanup: delete DC + bitmap
```

This happens at ~0.5ms per frame on modern hardware. The background thread does this at 250Hz (every 4ms).

---

## Known Scenarios Where Color Can Be Wrong

[Same as before — unchanged]

---

## Known Issues — Realistic Fixes (Backlog)

[Same as before — unchanged]

### Crate Versions Used

All latest stable as of June 2026.
