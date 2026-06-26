# Color Picker — Production Build Plan

## User Flow (The Real Interaction)

```
Tool runs as system tray daemon (background, small RAM, no window).

1. You're in your editor/design tool/browser
   → See a color you want to capture

2. Press and hold ALT
   → Small overlay appears ~20px below cursor
   → Shows zoomed pixel grid + hex color + color name
   → Overlay stays visible as long as ALT is held

3. Move mouse while holding ALT
   → Overlay follows cursor smoothly
   → Color values update in real-time
   → You can scan across the screen seeing live color data

4. Left click (while still holding ALT)
   → Color copies to clipboard (hex by default)
   → Overlay dismisses immediately
   → You can release ALT now

5. Ctrl+V → #FF5733 appears in your editor

6. Next time you need a color → press ALT again → same flow

Optional variations (configurable):
  - Press H while holding ALT → toggle history panel
  - Press Space while holding ALT → freeze/lock the color for detailed inspection
  - Release ALT without clicking → nothing copied, overlay just disappears
  - Right click or Esc → dismiss without copying
```

---

## Tech Stack (Corrected)

| Layer | Choice | Why |
|---|---|---|
| **Language** | Rust | Zero GC pauses, tiny binary (~2-5 MB), native OS API access |
| **Windowing** | `winit` | De facto Rust windowing, raw window handles for platform overlay config |
| **Rendering** | `softbuffer` (framebuffer only) | ~250×180px overlay of zoomed pixels + text + swatches doesn't need GPU. Drop wgpu. |
| **Screen Capture** | Platform-native via thin Rust wrappers | See table below. |
| **Clipboard** | `arboard` | Pure Rust, cross-platform |
| **Color Math** | `palette` | sRGB, Lab, LCh, delta-E, harmonies |
| **Config** | `serde` + TOML | Standard for Rust CLI tools |
| **Hotkeys** | Platform-native APIs (not `rdev`) | `GetAsyncKeyState` (Win), `CGEventTap` (macOS), `XQueryKeymap` (Linux) |
| **ALT key tracking** | Polling thread + main loop | 250Hz polling on background thread, results fed via channel |
| **Error handling** | `thiserror` + `color-eyre` | Applied from M1, not retrofitted |
| **System tray** | `tray-icon` crate | Cross-platform, active maintenance |

### Screen Capture Per Platform

| Platform | API | Why |
|---|---|---|
| **Windows** | `BitBlt` via `CreateCompatibleDC` | 96×96 region in <0.5ms. DXGI is overkill for color picking. |
| **macOS** | `CGDisplayStream` via `objc2` bindings | Direct CoreGraphics.framework calls. `core-graphics` crate is unmaintained. |
| **Linux/X11** | `XShmGetImage` | Shared memory, fast and mature. |
| **Linux/Wayland** | XWayland fallback (v1.0) | PipeWire portal is fragile. Add native support when stable. |

---

## Core Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Background Thread                         │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │ ALT Key      │  │ Cursor       │  │ Screen Capture     │  │
│  │ Poller       │  │ Poller       │  │ (96×96 region)     │  │
│  │ (GetAsyncKey │  │ (GetCursorPos│  │                    │  │
│  │ State loop)  │  │  polling)    │  │                    │  │
│  └──────┬───────┘  └──────┬───────┘  └─────────┬──────────┘  │
│         │                 │                    │             │
│         └─────────┬───────┴────────────┐       │             │
│                   │   Channel::Message │       │             │
└───────────────────┼────────────────────┼───────┘             │
                    ▼                    ▼                     │
┌──────────────────────────────────────────────────────────────┐│
│                    Main Thread (winit Event Loop)             ││
│  ┌─────────────────────────────────────────────────────────┐ ││
│  │                  App State                               │ ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │ ││
│  │  │ State    │  │ Pixel    │  │ History  │  │ Config │  │ ││
│  │  │ Machine  │  │ Analyzer │  │ Store    │  │        │  │ ││
│  │  │(idle/active)│          │  │          │  │        │  │ ││
│  │  └──────────┘  └──────────┘  └──────────┘  └────────┘  │ ││
│  │  ┌──────────┐  ┌──────────┐  ┌────────────────────────┐ │ ││
│  │  │ Overlay  │  │ Color    │  │ Tray Icon Handler      │ │ ││
│  │  │ Renderer │  │ Math     │  │ (system notifications) │ │ ││
│  │  └──────────┘  └──────────┘  └────────────────────────┘ │ ││
│  └─────────────────────────────────────────────────────────┘ ││
└──────────────────────────────────────────────────────────────┘│
```

### State Machine

```
                    ALT press
  ┌──────┐  ───────────────→  ┌─────────┐
  │ IDLE │                    │ ACTIVE  │
  │      │  ←──────────────  │         │
  └──────┘  ALT release       └─────────┘
                                │     │
                                │     │
                        ┌───────┘     └───────┐
                        ▼                     ▼
                  ┌──────────┐          ┌──────────┐
                  │ COPIED   │          │ LOCKED   │
                  │ (clicked)│          │ (Space)  │
                  │→ back to │          │→ back to │
                  │  IDLE    │          │  ACTIVE  │
                  └──────────┘          └──────────┘
```

### Data Flow Per Frame (Active Mode)

```
Background Thread (250Hz polling):
  1. Check GetAsyncKeyState(VK_MENU) — is ALT held?
  2. If ALT held: GetCursorPos → (x, y)
  3. If position changed: BitBlt(x-48, y-48, 96, 96) → RGBA buffer
  4. Send (cursor_x, cursor_y, rgba_buffer) to main thread via channel

Main Thread (winit RedrawRequested):
  1. Receive latest frame from channel (or skip if none)
  2. PixelAnalyzer::sample_center(buffer) → Color { r, g, b, a }
  3. ColorMath: format as hex, RGB, HSL → cache strings
  4. OverlayRenderer: composite framebuffer (zoom loupe + text + swatches)
  5. softbuffer: present framebuffer to window
  6. Reposition window to (cursor_x - overlay_w/2, cursor_y - overlay_h - 20)
```

### Event Loop Architecture (Pseudocode)

```rust
fn main() {
    let event_loop = EventLoop::new();
    let (frame_tx, frame_rx) = channel::<(i32, i32, Vec<u8>)>();

    // Spawn background polling thread
    std::thread::spawn(move || {
        let capture = ScreenCapture::new();
        loop {
            if is_alt_held() {
                let (x, y) = get_cursor_pos();
                let buf = capture.grab_region(x - 48, y - 48, 96, 96);
                frame_tx.send((x, y, buf)).ok();
            }
            std::thread::sleep(Duration::from_millis(4)); // ~250Hz
        }
    });

    // Main thread: winit event loop
    event_loop.run(|event, window, control_flow| {
        control_flow.set_poll();

        match event {
            Event::RedrawRequested => {
                if let Ok((cx, cy, buf)) = frame_rx.try_recv() {
                    let color = analyze_pixel(&buf);
                    let overlay = render_overlay(&buf, &color);
                    window.set_outer_position(...);
                    softbuffer.present(&overlay);
                } else if !is_active {
                    window.set_visible(false);
                }
            }
            _ => {}
        }
    });
}
```

---

## Overlay UI Layout

```
┌──────────────────────────────┐
│  ┌──────┐                    │
│  │ 4×   │  #FF5733          │
│  │ zoom │  rgb(255,87,51)   │
│  │ loupe│  hsl(11,100%,60%) │
│  │      │  ── Vermilion     │  ← color name
│  └──────┘  ┌──┐             │
│            │██│             │  ← live swatch
│  ──────────                 │
│  Recent: ■ #3344FF ■ #33FF57│  ← history strip
│                              │
│  [click to copy]             │
└──────────────────────────────┘
     ~250px × ~200px
```

---

## Milestones (Corrected)

### M1: Scaffold + Daemon Core + Screen Capture (~1 week)
**Deliverable:** System tray daemon running. Press ALT → prints hex of pixel under cursor.

- Cargo workspace: `screen_capture/`, `color.rs`, `clipboard.rs`, `hotkey.rs`, `config.rs`, `error.rs`
- `ScreenCapture` trait + platform impls (BitBlt, CGDisplay, XShmGetImage)
- `PixelAnalyzer` → `Color { r, g, b, a }`
- `HotkeyPoller`: ALT key detection per platform
- System tray icon via `tray-icon` crate
- Background thread: 250Hz ALT polling + cursor/capture loop
- CLI mode: `pick --once` captures and prints hex
- `thiserror` + `color-eyre` from day one — **no unwrap()** on OS API calls
- State machine: IDLE ↔ ACTIVE ↔ IDLE

### M2: System Tray + ALT Listener (~3-4 days)
**Deliverable:** Tray icon with menu. ALT press/release drives capture.

- System tray context menu: Pick Color, Settings, History, Quit
- ALT down → start cursor tracking + capture, store in ring buffer
- ALT up → stop capture, overlay hidden
- In-memory ring buffer for color history
- Config file at XDG_CONFIG_HOME/pick/config.toml (basic fields)

### M3: Overlay Window (~1 week)
**Deliverable:** Transparent overlay near cursor when ALT held, follows at 60 FPS.

- winit: transparent, always-on-top, click-through, no focus
- OS-specific window properties (`WS_EX_TRANSPARENT`, `.ignoresMouseEvents`, `_NET_WM_STATE` variants)
- Window show/hide tied to ALT state (IDLE = hidden, ACTIVE = visible)
- `ControlFlow::Poll` + `RedrawRequested` for frame sync
- softbuffer framebuffer (transparent RGBA)
- Screen edge detection (flip above if near bottom)

### M4: Overlay UI Rendering (~1 week)
**Deliverable:** Full overlay: zoom loupe, color values, color name, swatches.

- Zoom loupe: nearest-neighbor 4× with pixel grid overlay
- Text: `ab_glyph` + embedded JetBrains Mono subset
- Color info: hex (primary), RGB + HSL (secondary), color name
- Live swatch filled with current color
- History strip (last 5 picks)
- Copy animation (200ms checkmark flash)

### M5: Input Handling + Lock Mode + Config (~1 week)
**Deliverable:** Complete interaction model with config file.

- Left click → copy and dismiss (ACTIVE → IDLE)
- Right click / Esc → dismiss without copy
- Space → LOCKED state (freeze cursor position)
- H → toggle history panel
- 1-9 → quick format switch
- Full config: default_format, zoom_level, copy_on_click, theme, history_size
- macOS permission denied detection + help dialog
- Error handling: remote desktop, locked desktop, permission errors

### M6: Harmonies + Advanced Features (~3-4 days)
**Deliverable:** Professional color tools in overlay.

- `palette` crate: complementary, triadic, tetradic, analogous, monochromatic
- Harmony display: 5 swatches with labels
- Averaged sampling (3×3, 5×5 pixel average)
- WCAG contrast ratio (AA/AAA pass/fail)
- Lock mode crosshair for precise pixel targeting

### M7: Polish + Distribution (~1 week)
**Deliverable:** Signed, packaged installers for all platforms.

- DPI scaling (125%, 150%, 200%, per-monitor)
- Multi-monitor: per-display capture context
- Frame time <16ms, CPU <2% at idle
- Auto-start with OS boot (configurable)
- CI: GitHub Actions for Win/macOS/Linux
- Installers: MSI (Win), signed DMG (macOS), deb/rpm/AppImage (Linux)
- Auto-update: GitHub releases check

---

## What Changed From Original Plan

| Change | Original | Corrected | Why |
|---|---|---|---|
| **Hotkeys** | `rdev` crate | Platform-native APIs | rdev unmaintained, causes input lag |
| **Rendering** | `wgpu` + `softbuffer` | `softbuffer` only | wgpu is overkill for a 250px overlay |
| **Screen capture** | DXGI (Win) | BitBlt (Win) | BitBlt faster for small region reads |
| **macOS bindings** | `core-graphics` crate | `objc2` + direct calls | core-graphics unmaintained since 2020 |
| **Daemon architecture** | "Phase 3" | M1 (foundational) | Flow requires always-running daemon |
| **Activation** | Ctrl+Shift+C chorded | **ALT key (hold)** | Simpler, one-key, hold-to-show |
| **Dismissal** | Click only | ALT release + click | Natural: release ALT when done, click to copy |
| **Error handling** | "M7: zero unwrap" | `thiserror` from M1 | Every OS call can fail, handle immediately |
| **System tray** | "Phase 3" | M1 (core) | Tray icon is the daemon UI, needed from start |

---

## Open Decisions

| Decision | Options | Recommendation |
|---|---|---|
| Wayland support | Full PipeWire vs XWayland-only | XWayland-only for v1.0 |
| System tray library | `tray-icon` vs `tao-tray` vs manual | `tray-icon` (cross-platform, maintained) |
| macOS permission UX | Dialog vs System Settings link | Both: dialog + deep link |
| Auto-start | Registry/LaunchAgents vs skip | Add in M7 |
| Polling rate | 120Hz vs 250Hz vs 500Hz | **250Hz (4ms)** — enough for human interaction |
| Open source | MIT/Apache2 vs proprietary | MIT — community trust, package manager inclusion |
| Binary name | `pick` may be taken on crates.io | Alts: `cursor-pick`, `screenpick`, `pixel-pick` |

---

## Key Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| ALT key conflicts with app menus | High | Medium | `GetAsyncKeyState` reads physical key, bypasses input processing |
| macOS screen recording permission | High | High | Native dialog with System Settings deep link |
| macOS 14+ still requires permission | High | Medium | Document clearly. No workaround. |
| Windows high-contrast mode | Medium | Low | Detect via SPI, switch overlay palette |
| RDP / remote desktop capture fails | Medium | Low | Detect remote session, clear error message |
| ALT polling CPU usage | Low | Medium | `sleep(4ms)` keeps it at ~1% CPU |
| Anti-virus false positive | Low | High | Code signing, open source reputation |

---

## Testing Strategy

- **Unit tests**: pixel analysis, color math, harmonies, config parsing
- **Integration tests**: capture returns RGBA, clipboard round-trip (mock OS APIs for CI)
- **Snapshot tests**: overlay rendering at various zooms (`insta` image diffs)
- **Platform CI**: build + smoke on Windows, macOS, Ubuntu, Fedora
- **Performance benchmarks**: frame time, pixel read latency, memory (`criterion`)
- **Manual QA**: clean install on each OS, test all input combos

---

## Estimated Effort

- M1–M2 (daemon + capture + tray): ~1.5 weeks
- M3–M4 (overlay + rendering): ~2 weeks
- M5–M6 (input + harmonies): ~1.5 weeks
- M7 (packaging + distribution): ~1 week
- **Total to v1.0: ~5–7 weeks** (single experienced Rust dev)
