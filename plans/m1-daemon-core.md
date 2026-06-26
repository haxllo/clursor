# M1: Daemon Core + Screen Capture Plan

## Goal
System tray daemon running in background. `pick --once` prints hex under cursor. `pick` (no args) starts daemon → ALT key → logs pixel color.

## Crate Versions (Latest as of June 2026)

| Crate | Version | Purpose |
|---|---|---|
| `windows-sys` | 0.61.2 | Win: BitBlt, GetAsyncKeyState, GetCursorPos |
| `objc2` | 0.6.4 | macOS: CoreGraphics bindings |
| `objc2-foundation` | 0.3.2 | macOS: NS types |
| `objc2-core-graphics` | 0.3.2 | macOS: CGDisplay APIs |
| `x11rb` | 0.13.0 | Linux: XShmGetImage |
| `arboard` | 3.6.1 | Clipboard |
| `serde` / `toml` | 1.0.228 / 1.1.2 | Config serialization |
| `serde_json` | 1.x | History persistence |
| `color-eyre` | 0.6.5 | Error reporting |
| `thiserror` | 2.0.18 | Error types |
| `tracing` | 0.1.44 | Structured logging |
| `tracing-subscriber` | 0.3.19 | Log output |
| `clap` | 4.x | CLI arg parsing |
| `tray-icon` | 0.24.1 | System tray |
| `muda` | 0.19.1 | Tray context menu |
| `image` | 0.25.x | Tray icon generation |
| `palette` | 0.7.6 | Color math (minimal in M1) |

## Module Structure

```
src/
├── main.rs              # Entry: --once → capture & print, no args → daemon
├── capture/
│   ├── mod.rs           # ScreenCapture trait + factory
│   ├── windows.rs       # BitBlt impl
│   ├── macos.rs         # CGDisplay impl (stub for now on Windows)
│   └── linux.rs         # XShmGetImage impl (stub for now)
├── color.rs             # Color {r,g,b,a}, PixelAnalyzer, format as hex/rgb/hsl
├── hotkey.rs            # Platform::is_alt_held()
├── daemon.rs            # System tray + background poll loop + state machine
├── config.rs            # Config struct, load/save TOML
├── clipboard.rs         # arboard wrapper
└── error.rs             # AppError enum, Result<T>
```

## Implementation Order

### Step 1: Project scaffold + error.rs + color.rs + config.rs
- `cargo init pick`
- Set up Cargo.toml with deps
- `error.rs`: AppError enum with thiserror + color-eyre hook
- `color.rs`: Color struct with formatting (hex, rgb, hsl)
- `config.rs`: Config default + TOML file I/O

### Step 2: capture module (Windows first)
- `capture/mod.rs`: `ScreenCapture` trait with `grab_region(x, y, w, h) -> Result<Vec<u8>>`
- `capture/windows.rs`: BitBlt from CreateCompatibleDC → read pixels
- `capture/macos.rs`: #![cfg(target_os = "macos")] stub
- `capture/linux.rs`: #![cfg(target_os = "linux")] stub
- Factory: `ScreenCapture::new() -> Result<Box<dyn ScreenCapture>>`

### Step 3: CLI mode
- `main.rs`: clap CLI with `--once` flag
- `pick --once`: capture screen → sample center pixel → print hex to stdout
- Test: run binary, verify output

### Step 4: hotkey module
- `hotkey.rs`: platform-specific ALT key detection
- Windows: `GetAsyncKeyState(VK_MENU) & 0x8000 != 0`
- macOS: CGEventTap (stub for now)
- Linux: XQueryKeymap (stub for now)

### Step 5: daemon mode + system tray
- `daemon.rs`: background thread polling ALT at 250Hz
- On ALT press: capture pixel, log with tracing
- System tray icon via tray-icon crate
- Tray context menu: "Pick Color" (simulate ALT), "Settings" (stub), "Quit"
- daemon.run() is the main entry point when no --once flag

## Key Decisions for M1

| Decision | Choice | Why |
|---|---|---|
| Platform first | Windows | We're on Windows. Mac/Linux stubs for now |
| Capture region | 96×96 pixels | Small enough for <1ms capture, large enough for zoom loupe (M3) |
| ALT polling rate | 250Hz (4ms) | Responsive enough for human interaction, low CPU |
| Tray icon | Generated programmatically (colored circle) | No external asset files needed |
| Config path | XDG_CONFIG_HOME/pick/config.toml | Standard cross-platform convention |
| Error handling | thiserror + color-eyre | From day one, no unwrap on OS calls |

## Risks

| Risk | Mitigation |
|---|---|
| windows-sys 0.61 feature mismatch | Pin exact features: Gdi, WindowsAndMessaging, Foundation |
| tray-icon event loop model | tray-icon uses crossbeam-channel, push-based from background thread |
| BitBlt returns black on DWM/high-contrast | Fallback message but acceptable for v1 |
| `pick` binary name conflict | Check crates.io. If taken, rename before publishing |
