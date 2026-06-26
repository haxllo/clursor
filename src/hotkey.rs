/// Platform-specific ALT key state detection.

/// Returns true if the ALT key is currently being held down.
pub fn is_alt_held() -> bool {
    #[cfg(target_os = "windows")]
    {
        is_alt_held_windows()
    }
    #[cfg(target_os = "macos")]
    {
        is_alt_held_macos()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        is_alt_held_linux()
    }
}

#[cfg(target_os = "windows")]
fn is_alt_held_windows() -> bool {
    // VK_MENU = 0x12. GetAsyncKeyState checks physical key state (0x8000 = current press).
    // This reads raw hardware state, bypassing app-level menu processing.
    const VK_MENU: i32 = 0x12;
    #[link(name = "user32")]
    extern "system" {
        fn GetAsyncKeyState(vKey: i32) -> i16;
    }
    unsafe { GetAsyncKeyState(VK_MENU) as u16 & 0x8000 != 0 }
}

#[cfg(target_os = "macos")]
fn is_alt_held_macos() -> bool {
    false
}

#[cfg(all(unix, not(target_os = "macos")))]
fn is_alt_held_linux() -> bool {
    false
}
