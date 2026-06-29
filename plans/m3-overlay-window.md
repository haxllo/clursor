# M3: Overlay Window

## Goal
Transparent overlay appears near cursor when Ctrl held, follows at 60 FPS, disappears on release. No UI content yet (that's M4) — just validates the window positioning pipeline.

## What Gets Built

| Component | Description |
|---|---|
| **Overlay window** | winit window: no decorations, always-on-top, hidden by default |
| **Platform styles** | Windows: `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST` |
| **Visibility toggle** | Show on Ctrl press (Idle→Active), hide on Ctrl release (Active→Idle) |
| **Cursor tracking** | Reposition window to (cursor + 12px offset) on each `AboutToWait` frame |
| **Screen edge flip** | If cursor near screen bottom, position above instead of below |
| **Rounded corners** | `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND)` — DWM smooth anti-aliased corners |

## What M3 Does NOT Do
- No zoom loupe, no text, no swatches (M4)
- No click-through (`WS_EX_TRANSPARENT`) — requires per-pixel alpha rendering (future)
- No SetWindowRgn — DWM handles rounding natively with anti-aliasing

## Implementation

### daemon.rs changes
1. Create overlay window at startup (hidden)
2. Get HWND via winit, set platform styles + rounded corners
3. In state transitions: toggle visibility
4. In AboutToWait (Active): reposition + request_redraw
5. In WindowEvent::RedrawRequested (overlay): call renderer.paint()

### New file: `renderer.rs`
GDI rendering pipeline (see M4 spec).

### New file: `overlay.rs`
`hwnd_from_window()`, `apply_platform_styles()`, `apply_rounded_corners()`, `position_near_cursor()`.

## Verification
1. `cargo run` → daemon starts
2. Hold Ctrl → overlay appears near cursor, follows smoothly
3. Release Ctrl → overlay disappears
4. No focus steal (no taskbar flash, no menu activation)
5. Near screen edges → overlay flips to stay visible
