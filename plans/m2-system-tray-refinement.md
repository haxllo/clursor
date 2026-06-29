# M2: System Tray + Ctrl Listener Refinement

## Goal
Daemon is fully interactive: tray click picks color, Ctrl hold → logs real-time, Ctrl release → copies last color to clipboard, tray tooltip shows last pick.

## What Changed From M1 Plan

M1's daemon captures and logs but doesn't interact. M2 wires up:
- Ctrl release triggers clipboard copy
- Tray click triggers a "pick" (capture + copy like Ctrl without holding)
- Tooltip shows last picked color
- Config.format controls output format
- Structured state machine for debugging

## What M2 Does NOT Do
- No overlay window (that's M3)
- No visual feedback beyond tooltip/logs
- No lock mode or advanced inputs

---

## Refinement Steps

### Step 1: Clean Ctrl API for windows-sys
Replace `extern "system"` in hotkey.rs with proper `windows-sys` binding. Use:
```rust
windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState
```
This removes the manual linker declaration.

### Step 2: AppState Machine
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    Idle,
    Active,  // Ctrl is held, capturing
}
```

The daemon tracks `ctrl_held` as an `AtomicBool` (already done). Add explicit state transitions with tracing events:
```
Idle → Ctrl press → Active (log: "picker activated")
Active → Ctrl release → Idle (log: "picker deactivated, last: #HEX")
```

### Step 3: Tray Click → "Pick"
When tray icon is clicked:
1. Capture pixel at current cursor
2. Copy to clipboard using config.default_format
3. Update tooltip with last picked color
4. Log the pick

This gives users a single-click way to pick without holding Ctrl.

### Step 4: Ctrl Release → Copy
On Ctrl release (Active → Idle transition):
- Copy the last captured color to clipboard
- Update tray tooltip
- Log the pick

Currently M1 just logs on capture frames while Ctrl is held. M2 copies on release.

### Step 5: Config Integration
- Read config.default_format on daemon start
- Use it for: CLI output (`--once`), clipboard copy, tray tooltip

Add a `format_color(color, format) -> String` helper.

### Step 6: Settings Stub
The "Settings..." menu item currently does nothing. M2 opens the config file in the default text editor (or shows a message about where it lives).

---

## New/Changed Files

```
src/
├── hotkey.rs            # CHANGED: use windows-sys for GetAsyncKeyState
├── daemon.rs            # CHANGED: state machine, tray click, Ctrl release copy
├── config.rs            # CHANGED: add format_color helper
├── clipboard.rs         # UNCHANGED (already ready, now called from daemon)
├── main.rs              # CHANGED: --once respects config.default_format
```

---

## Edge Cases

| Case | Behavior |
|---|---|
| Ctrl pressed, no mouse movement | Capture once, wait. Release → copy that color |
| Ctrl tapped (press+release same frame) | Capture and copy immediately |
| Tray double-click | Same as single click (capture + copy) |
| Ctrl held, tray menu opened by another click | Ctrl capture continues independently |
| Config file missing or corrupt | Default to hex format, log warning |
| Clipboard unavailable | Log error, don't crash |

---

## Verification

1. `cargo run` → tray icon appears
2. Click tray icon → color copied, tooltip updates
3. Hold Ctrl, move mouse → tracing shows live captures
4. Release Ctrl → last color copied, tooltip updates
5. Settings... → opens config file or shows path
6. Quit → clean shutdown
7. `cargo run -- --once` → respects config format
8. Kill daemon, restart → no state leaks
