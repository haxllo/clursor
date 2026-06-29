use palette::color_difference::Ciede2000;
use palette::{FromColor, Lab, Srgb};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn to_rgb_string(&self) -> String {
        format!("rgb({},{},{})", self.r, self.g, self.b)
    }

    pub fn to_hsl_string(&self) -> String {
        let srgb = Srgb::new(self.r, self.g, self.b);
        let lin: palette::LinSrgb = srgb.into_linear();
        let hsl = palette::Hsl::from_color(lin);
        let h = (hsl.hue.into_positive_degrees().round() as u16) % 360;
        let s = (hsl.saturation * 100.0).round() as u8;
        let l = (hsl.lightness * 100.0).round() as u8;
        format!("hsl({h},{s},{l}%)")
    }

    pub fn name(&self) -> &'static str {
        if crate::xkcd_data::XKCD.is_empty() {
            return "Unknown";
        }

        let srgb = Srgb::new(self.r, self.g, self.b);
        let lin: palette::LinSrgb = srgb.into_linear();
        let lab: Lab = Lab::from_color(lin);

        let mut best_name = "Unknown";
        let mut best_delta = f32::MAX;

        for &(name, (r, g, b)) in crate::xkcd_data::XKCD {
            let target_srgb = Srgb::new(r, g, b);
            let target_lin: palette::LinSrgb = target_srgb.into_linear();
            let target_lab: Lab = Lab::from_color(target_lin);
            let delta = lab.difference(target_lab);
            if delta < best_delta {
                best_delta = delta;
                best_name = name;
            }
        }

        best_name
    }
}

pub struct PixelAnalyzer;

impl PixelAnalyzer {
    pub fn sample_center(buf: &[u8], width: u32, height: u32) -> Color {
        let cx = (width / 2) as usize;
        let cy = (height / 2) as usize;
        let idx = (cy * width as usize + cx) * 4;
        Color::new(buf[idx], buf[idx + 1], buf[idx + 2])
    }

    #[allow(dead_code)]
    pub fn sample_averaged(buf: &[u8], width: u32, height: u32, size: u32) -> Color {
        let cx = (width / 2) as i32;
        let cy = (height / 2) as i32;
        let half = (size / 2) as i32;
        let mut r_sum: u64 = 0;
        let mut g_sum: u64 = 0;
        let mut b_sum: u64 = 0;
        let mut count: u64 = 0;

        let w = width as i32;
        for dy in -half..=half {
            for dx in -half..=half {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < w && py >= 0 && py < height as i32 {
                    let idx = (py * w + px) as usize * 4;
                    r_sum += buf[idx] as u64;
                    g_sum += buf[idx + 1] as u64;
                    b_sum += buf[idx + 2] as u64;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Self::sample_center(buf, width, height);
        }
        Color::new(
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
        )
    }
}
