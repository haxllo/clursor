use std::ffi::c_void;

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen,
    CreateSolidBrush, DeleteDC,
    DeleteObject, FillRect, GetDC, GetStockObject, LineTo, MoveToEx, ReleaseDC, RoundRect,
    SelectObject, SetBkMode, SetROP2, SetTextColor, StretchDIBits, TextOutW,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, PS_SOLID, R2_COPYPEN, R2_NOT,
    SRCCOPY, TRANSPARENT,
};

use crate::color::Color;
use crate::overlay::{OVERLAY_H, OVERLAY_W};

// Layout — 280×190 window
const PAD: i32 = 12;
const LOUPE_DST: i32 = 90;
const LOUPE_SRC: i32 = 5;
const ZOOM: i32 = 18;
const LOUPE_X: i32 = PAD;
const LOUPE_Y: i32 = 14;
const TEXT_X: i32 = LOUPE_X + LOUPE_DST + 14;
const SWATCH_Y: i32 = LOUPE_Y + LOUPE_DST + PAD;
const SWATCH_H: i32 = 20;


fn gdi_color(r: u8, g: u8, b: u8) -> u32 {
    (b as u32) << 16 | (g as u32) << 8 | (r as u32)
}

pub struct Renderer {
    bgra_buf: Vec<u8>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            bgra_buf: vec![0u8; 96 * 96 * 4],
        }
    }

    pub fn update_capture(&mut self, rgba_buf: &[u8]) {
        if rgba_buf.len() < 96 * 96 * 4 { return; }
        self.bgra_buf[..rgba_buf.len()].copy_from_slice(rgba_buf);
        for p in self.bgra_buf.chunks_exact_mut(4) { p.swap(0, 2); }
    }

    pub fn paint(&self, window: &winit::window::Window, color: Color, name: &str) {
        let Some(hwnd) = crate::overlay::hwnd_from_window(window) else { return };

        unsafe {
            let hdc = GetDC(hwnd);
            if hdc.is_null() { return; }

            let mem = CreateCompatibleDC(hdc);
            let bmp = CreateCompatibleBitmap(hdc, OVERLAY_W, OVERLAY_H);
            let old_bmp = SelectObject(mem, bmp);

            let saved_rop = SetROP2(mem, R2_COPYPEN);
            self.render_background(mem);
            self.render_loupe(mem);
            self.render_grid(mem);
            self.render_swatch(mem, color);
            self.render_text(mem, color, name);
            SetROP2(mem, saved_rop);

            BitBlt(hdc, 0, 0, OVERLAY_W, OVERLAY_H, mem, 0, 0, SRCCOPY);

            SelectObject(mem, old_bmp);
            DeleteObject(bmp);
            DeleteDC(mem);
            ReleaseDC(hwnd, hdc);
        }
    }

    unsafe fn render_background(&self, hdc: *mut c_void) {
        let bg = gdi_color(22, 22, 42);
        let brush = CreateSolidBrush(bg);
        let r = RECT { left: 0, top: 0, right: OVERLAY_W, bottom: OVERLAY_H };
        FillRect(hdc, &r, brush);
        DeleteObject(brush);
    }

    unsafe fn render_loupe(&self, hdc: *mut c_void) {
        let center = 48i32;
        let s = center - LOUPE_SRC / 2;

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = 96;
        bmi.bmiHeader.biHeight = -96;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        StretchDIBits(hdc, LOUPE_X, LOUPE_Y, LOUPE_DST, LOUPE_DST,
            s, s, LOUPE_SRC, LOUPE_SRC,
            self.bgra_buf.as_ptr() as _, &bmi, DIB_RGB_COLORS, SRCCOPY);
    }

    unsafe fn render_grid(&self, hdc: *mut c_void) {
        let pen = CreatePen(PS_SOLID, 1, gdi_color(50, 50, 65));
        let old = SelectObject(hdc, pen as _);

        for i in 0..=LOUPE_SRC {
            let o = i * ZOOM;
            MoveToEx(hdc, LOUPE_X + o, LOUPE_Y, std::ptr::null_mut());
            LineTo(hdc, LOUPE_X + o, LOUPE_Y + LOUPE_DST);
            MoveToEx(hdc, LOUPE_X, LOUPE_Y + o, std::ptr::null_mut());
            LineTo(hdc, LOUPE_X + LOUPE_DST, LOUPE_Y + o);
        }
        SelectObject(hdc, old);
        DeleteObject(pen);

        // Crosshair — gap-center, negative (inverts pixels for visibility)
        let cx = LOUPE_X + LOUPE_DST / 2;
        let cy = LOUPE_Y + LOUPE_DST / 2;
        let gap = 4;
        let arm = 10;

        let prev_rop = SetROP2(hdc, R2_NOT);
        let c = CreatePen(PS_SOLID, 1, 0);
        let o2 = SelectObject(hdc, c as _);
        MoveToEx(hdc, cx - gap - arm, cy, std::ptr::null_mut());
        LineTo(hdc, cx - gap, cy);
        MoveToEx(hdc, cx + gap, cy, std::ptr::null_mut());
        LineTo(hdc, cx + gap + arm, cy);
        MoveToEx(hdc, cx, cy - gap - arm, std::ptr::null_mut());
        LineTo(hdc, cx, cy - gap);
        MoveToEx(hdc, cx, cy + gap, std::ptr::null_mut());
        LineTo(hdc, cx, cy + gap + arm);
        SelectObject(hdc, o2);
        DeleteObject(c);
        SetROP2(hdc, prev_rop);
    }

    unsafe fn render_swatch(&self, hdc: *mut c_void, color: Color) {
        let brush = CreateSolidBrush(gdi_color(color.r, color.g, color.b));
        let r = RECT {
            left: PAD,
            top: SWATCH_Y,
            right: OVERLAY_W - PAD,
            bottom: SWATCH_Y + SWATCH_H,
        };
        FillRect(hdc, &r, brush);
        DeleteObject(brush);

        let pen = CreatePen(PS_SOLID, 1, gdi_color(60, 60, 80));
        let old_pen = SelectObject(hdc, pen as _);
        let hollow = GetStockObject(5); // NULL_BRUSH — prevent RoundRect from filling
        let old_brush = SelectObject(hdc, hollow);
        RoundRect(hdc,
            PAD - 1, SWATCH_Y - 1,
            OVERLAY_W - PAD + 1, SWATCH_Y + SWATCH_H + 1,
            4, 4);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        DeleteObject(pen);
    }

    unsafe fn render_text(&self, hdc: *mut c_void, color: Color, name: &str) {
        SetBkMode(hdc, TRANSPARENT as i32);

        let font_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();

        // Hex — bold 22px, white
        let hex_font = CreateFontW(
            22, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr(),
        );
        let of = SelectObject(hdc, hex_font as _);
        SetTextColor(hdc, gdi_color(255, 255, 255));
        let hex = color.to_hex();
        let w: Vec<u16> = hex.encode_utf16().collect();
        TextOutW(hdc, TEXT_X, 14, w.as_ptr(), w.len() as i32);
        SelectObject(hdc, of);
        DeleteObject(hex_font);

        // Body — 15px regular, gray
        let body = CreateFontW(
            15, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr(),
        );
        let ob = SelectObject(hdc, body as _);

        let gray = gdi_color(160, 160, 175);
        SetTextColor(hdc, gray);

        let rgb = color.to_rgb_string();
        let w: Vec<u16> = rgb.encode_utf16().collect();
        TextOutW(hdc, TEXT_X, 44, w.as_ptr(), w.len() as i32);

        let hsl = color.to_hsl_string();
        let w: Vec<u16> = hsl.encode_utf16().collect();
        TextOutW(hdc, TEXT_X, 62, w.as_ptr(), w.len() as i32);

        SelectObject(hdc, ob);
        DeleteObject(body);

        // Name — 15px italic, accent purple
        let name_font = CreateFontW(
            15, 0, 0, 0, 400, 1, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr(),
        );
        let on = SelectObject(hdc, name_font as _);
        SetTextColor(hdc, gdi_color(180, 150, 240));
        let w: Vec<u16> = name.encode_utf16().collect();
        TextOutW(hdc, TEXT_X, 82, w.as_ptr(), w.len() as i32);
        SelectObject(hdc, on);
        DeleteObject(name_font);
    }


}
