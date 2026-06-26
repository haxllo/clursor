# Progress — 2026-06-26

## Status

- **M1**: ✅ Complete — Daemon core + screen capture + system tray
- **M2**: ⬜ Not started — System tray + ALT refinement
- **M3**: ⬜ Not started — Overlay window
- **M4**: ⬜ Not started — Overlay UI rendering
- **M5**: ⬜ Not started — Input handling + lock mode + config
- **M6**: ⬜ Not started — Harmonies + advanced features
- **M7**: ⬜ Not started — Polish + distribution

## M1 Delivered

- `pick --once` captures pixel under cursor, prints `#HEX  Name`
- `pick` starts system tray daemon with purple circle icon
- ALT key polling on background thread at 250Hz
- Screen capture via BitBlt (Windows), stubs for macOS/Linux
- Color name lookup (130 named X11/web colors)
- Config file at `~/.config/pick/config.toml` (auto-created on first run)
- Tray menu: Settings... (stub) | Quit

## Previous Crate Versions Used

All latest stable as of June 2026.

## Next Steps (M2)

Refine the ALT listener interaction:
- Tray icon tooltip shows last picked color
- ALT hold → log with structured tracing events
- Edge detection logging (ALT pressed/released transitions)
- Prepare the capture/event pipeline for overlay (M3)
