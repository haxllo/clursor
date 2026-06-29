/// Platform-specific ALT key state detection.

/// Returns true if the Ctrl key is currently being held down.
pub fn is_ctrl_held() -> bool {
    #[cfg(target_os = "windows")]
    {
        is_ctrl_held_windows()
    }
    #[cfg(target_os = "macos")]
    {
        is_ctrl_held_macos()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        is_ctrl_held_linux()
    }
}

#[cfg(target_os = "windows")]
fn is_ctrl_held_windows() -> bool {
    // VK_CONTROL = 0x11. Ctrl does not trigger menu bars/flash focus
    // like ALT does, making it safe for hold-to-pick interaction.
    const VK_CONTROL: i32 = 0x11;
    #[link(name = "user32")]
    extern "system" {
        fn GetAsyncKeyState(vKey: i32) -> i16;
    }
    unsafe { GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000 != 0 }
}

#[cfg(target_os = "macos")]
fn is_ctrl_held_macos() -> bool {
    false
}

#[cfg(all(unix, not(target_os = "macos")))]
fn is_ctrl_held_linux() -> bool {
    false
}
