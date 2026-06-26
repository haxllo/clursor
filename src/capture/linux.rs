use crate::error::{AppError, Result};

pub struct LinuxCapturer;

impl LinuxCapturer {
    pub fn new() -> Result<Self> {
        Err(AppError::Capture("Linux capture not yet implemented".into()))
    }
}

impl super::ScreenCapture for LinuxCapturer {
    fn grab_region(&self, _x: i32, _y: i32, _width: u32, _height: u32) -> Result<Vec<u8>> {
        Err(AppError::Capture("Linux capture not yet implemented".into()))
    }
}
