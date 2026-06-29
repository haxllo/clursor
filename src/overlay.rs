use winit::window::Window;

pub const OVERLAY_W: i32 = 280;
pub const OVERLAY_H: i32 = 148;
pub const CURSOR_OFFSET: i32 = 12;

pub fn position_near_cursor(window: &Window, cursor_x: i32, cursor_y: i32) {
    let x = cursor_x + CURSOR_OFFSET;
    let mut y = cursor_y + CURSOR_OFFSET;

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CYVIRTUALSCREEN, SM_YVIRTUALSCREEN,
        };
        let screen_bottom =
            unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) + GetSystemMetrics(SM_CYVIRTUALSCREEN) };
        if y + OVERLAY_H > screen_bottom {
            y = cursor_y - CURSOR_OFFSET - OVERLAY_H;
        }
    }

    window.set_outer_position(winit::dpi::Position::Physical(
        winit::dpi::PhysicalPosition::new(x, y),
    ));
}

pub fn hwnd_from_window(window: &Window) -> Option<*mut core::ffi::c_void> {
    use winit::raw_window_handle::HasWindowHandle;
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        winit::raw_window_handle::RawWindowHandle::Win32(h) => {
            Some(h.hwnd.get() as *mut core::ffi::c_void)
        }
        _ => None,
    }
}

#[cfg(target_os = "windows")]
pub fn apply_platform_styles(window: &Window) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW,
        GWL_EXSTYLE, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    };

    let Some(hwnd) = hwnd_from_window(window) else { return };

    unsafe {
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            ex | WS_EX_NOACTIVATE as i32
               | WS_EX_TOOLWINDOW as i32
               | WS_EX_TOPMOST as i32,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_platform_styles(_window: &Window) {}

#[cfg(target_os = "windows")]
pub fn apply_rounded_corners(window: &Window) {
    let Some(hwnd) = hwnd_from_window(window) else { return };

    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: *mut core::ffi::c_void,
            dwAttribute: u32,
            pvAttribute: *const core::ffi::c_void,
            cbAttribute: u32,
        ) -> i32;
    }

    let preference = DWMWCP_ROUND;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_rounded_corners(_window: &Window) {}
