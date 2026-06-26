use std::mem;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
    GetDC, ReleaseDC, SelectObject, SRCCOPY, CAPTUREBLT, BI_RGB, DIB_RGB_COLORS,
    BITMAPINFO, BITMAPINFOHEADER, HDC,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
};

use crate::error::{AppError, Result};

pub struct WindowsCapturer {
    hdc_screen: HDC,
}

// HDC is a pointer but safe to send between threads on Windows
unsafe impl Send for WindowsCapturer {}

impl WindowsCapturer {
    pub fn new() -> Result<Self> {
        let hdc = unsafe { GetDC(std::ptr::null_mut()) };
        if hdc.is_null() {
            return Err(AppError::Capture(format!(
                "GetDC failed: {}",
                unsafe { GetLastError() }
            )));
        }
        Ok(Self { hdc_screen: hdc })
    }

    fn clamp_region(x: i32, y: i32, w: u32, h: u32) -> Option<(i32, i32, u32, u32)> {
        let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
        let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
        let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

        let rx = x.max(vx);
        let ry = y.max(vy);
        let rw = (w as i32 - (rx - x)).max(0).min(vw - (rx - vx));
        let rh = (h as i32 - (ry - y)).max(0).min(vh - (ry - vy));

        if rw <= 0 || rh <= 0 {
            return None;
        }
        Some((rx, ry, rw as u32, rh as u32))
    }
}

impl super::ScreenCapture for WindowsCapturer {
    fn grab_region(&self, x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>> {
        let (cx, cy, cw, ch) = Self::clamp_region(x, y, width, height)
            .ok_or_else(|| AppError::Capture("Capture region is outside screen bounds".into()))?;

        unsafe {
            let hdc_mem = CreateCompatibleDC(self.hdc_screen);
            if hdc_mem.is_null() {
                return Err(AppError::Capture(format!(
                    "CreateCompatibleDC failed: {}",
                    GetLastError()
                )));
            }

            let h_bitmap = CreateCompatibleBitmap(self.hdc_screen, cw as i32, ch as i32);
            if h_bitmap.is_null() {
                DeleteDC(hdc_mem);
                return Err(AppError::Capture(format!(
                    "CreateCompatibleBitmap failed: {}",
                    GetLastError()
                )));
            }

            // Select bitmap into memory DC (keep old object for cleanup)
            let old_bitmap = SelectObject(hdc_mem, h_bitmap as _);
            BitBlt(
                hdc_mem, 0, 0, cw as i32, ch as i32,
                self.hdc_screen, cx, cy,
                SRCCOPY | CAPTUREBLT,
            );

            let header_size = mem::size_of::<BITMAPINFOHEADER>() as u32;
            let mut bmi: BITMAPINFOHEADER = mem::zeroed();
            bmi.biSize = header_size;
            bmi.biWidth = cw as i32;
            bmi.biHeight = -(ch as i32); // top-down
            bmi.biPlanes = 1;
            bmi.biBitCount = 32;
            bmi.biCompression = BI_RGB;

            let pitch = cw as usize * 4;
            let mut buf = vec![0u8; pitch * ch as usize];

            let got = GetDIBits(
                hdc_mem,
                h_bitmap,
                0,
                ch,
                buf.as_mut_ptr() as _,
                &bmi as *const _ as *mut BITMAPINFO,
                DIB_RGB_COLORS,
            );

            // Cleanup
            SelectObject(hdc_mem, old_bitmap);
            DeleteObject(h_bitmap as _);
            DeleteDC(hdc_mem);

            if got == 0 {
                return Err(AppError::Capture(format!(
                    "GetDIBits failed: {}",
                    GetLastError()
                )));
            }

            // BGRA → RGBA
            for pixel in buf.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }

            // Pad with black if region was smaller than requested
            if cw != width || ch != height {
                let mut padded = vec![0u8; (width * height * 4) as usize];
                let dst_pitch = width as usize * 4;
                let src_pitch = cw as usize * 4;
                for row in 0..ch as usize {
                    let dst_off = row * dst_pitch;
                    let src_off = row * src_pitch;
                    padded[dst_off..dst_off + src_pitch]
                        .copy_from_slice(&buf[src_off..src_off + src_pitch]);
                }
                return Ok(padded);
            }

            Ok(buf)
        }
    }
}

impl Drop for WindowsCapturer {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(std::ptr::null_mut(), self.hdc_screen);
        }
    }
}
