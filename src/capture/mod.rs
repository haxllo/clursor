use crate::error::Result;

pub trait ScreenCapture: Send {
    /// Capture a rectangular region of the screen.
    /// Returns RGBA pixel data (row-major, top-left origin).
    fn grab_region(&self, x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>>;
}

pub fn create_capturer() -> Result<Box<dyn ScreenCapture>> {
    #[cfg(target_os = "windows")]
    {
        let c = windows::WindowsCapturer::new()?;
        Ok(Box::new(c))
    }
    #[cfg(target_os = "macos")]
    {
        let c = macos::MacosCapturer::new()?;
        Ok(Box::new(c))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let c = linux::LinuxCapturer::new()?;
        Ok(Box::new(c))
    }
}

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(all(unix, not(target_os = "macos")))]
pub mod linux;
