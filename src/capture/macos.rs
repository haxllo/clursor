use crate::error::{AppError, Result};

pub struct MacosCapturer;

impl MacosCapturer {
    pub fn new() -> Result<Self> {
        // TODO: Implement CGDisplayStream-based capture
        Err(AppError::Capture("macOS capture not yet implemented".into()))
    }
}

impl super::ScreenCapture for MacosCapturer {
    fn grab_region(&self, _x: i32, _y: i32, _width: u32, _height: u32) -> Result<Vec<u8>> {
        Err(AppError::Capture("macOS capture not yet implemented".into()))
    }
}
