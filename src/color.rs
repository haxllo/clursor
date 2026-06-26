use palette::FromColor;
use palette::{Srgb, Hsl};

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
        let hsl = Hsl::from_color(lin);
        let h = (hsl.hue.into_positive_degrees().round() as u16) % 360;
        let s = (hsl.saturation * 100.0).round() as u8;
        let l = (hsl.lightness * 100.0).round() as u8;
        format!("hsl({h},{s},{l}%)")
    }

    /// Find the closest named color by Euclidean distance in RGB space.
    pub fn name(&self) -> &'static str {
        let mut best = "Unknown";
        let mut best_dist = f64::MAX;

        for &(name, (r, g, b)) in NAMED_COLORS {
            let dr = self.r as f64 - r as f64;
            let dg = self.g as f64 - g as f64;
            let db = self.b as f64 - b as f64;
            let dist = dr * dr + dg * dg + db * db;
            if dist < best_dist {
                best_dist = dist;
                best = name;
            }
        }
        best
    }
}

pub struct PixelAnalyzer;

impl PixelAnalyzer {
    /// Sample the center pixel from an RGBA buffer.
    pub fn sample_center(buf: &[u8], width: u32, height: u32) -> Color {
        let cx = (width / 2) as usize;
        let cy = (height / 2) as usize;
        let idx = (cy * width as usize + cx) * 4;
        Color::new(buf[idx], buf[idx + 1], buf[idx + 2])
    }

    /// Sample an averaged region (size×size pixels) centered in the buffer.
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

/// Extended X11 / web color names (RGB tuples).
const NAMED_COLORS: &[(&str, (u8, u8, u8))] = &[
    ("Black", (0, 0, 0)),
    ("White", (255, 255, 255)),
    ("Red", (255, 0, 0)),
    ("Green", (0, 128, 0)),
    ("Blue", (0, 0, 255)),
    ("Yellow", (255, 255, 0)),
    ("Orange", (255, 165, 0)),
    ("Purple", (128, 0, 128)),
    ("Pink", (255, 192, 203)),
    ("Brown", (165, 42, 42)),
    ("Gray", (128, 128, 128)),
    ("Navy", (0, 0, 128)),
    ("Teal", (0, 128, 128)),
    ("Maroon", (128, 0, 0)),
    ("Lime", (0, 255, 0)),
    ("Aqua", (0, 255, 255)),
    ("Fuchsia", (255, 0, 255)),
    ("Silver", (192, 192, 192)),
    ("Coral", (255, 127, 80)),
    ("Crimson", (220, 20, 60)),
    ("Indigo", (75, 0, 130)),
    ("Khaki", (240, 230, 140)),
    ("Lavender", (230, 230, 250)),
    ("Magenta", (255, 0, 255)),
    ("Olive", (128, 128, 0)),
    ("Plum", (221, 160, 221)),
    ("Salmon", (250, 128, 114)),
    ("Tan", (210, 180, 140)),
    ("Tomato", (255, 99, 71)),
    ("Violet", (238, 130, 238)),
    ("Wheat", (245, 222, 179)),
    ("Azure", (240, 255, 255)),
    ("Beige", (245, 245, 220)),
    ("Bisque", (255, 228, 196)),
    ("BlanchedAlmond", (255, 235, 205)),
    ("Burlywood", (222, 184, 135)),
    ("Chartreuse", (127, 255, 0)),
    ("Chocolate", (210, 105, 30)),
    ("CornflowerBlue", (100, 149, 237)),
    ("Cornsilk", (255, 248, 220)),
    ("Cyan", (0, 255, 255)),
    ("DarkBlue", (0, 0, 139)),
    ("DarkCyan", (0, 139, 139)),
    ("DarkGray", (169, 169, 169)),
    ("DarkGreen", (0, 100, 0)),
    ("DarkKhaki", (189, 183, 107)),
    ("DarkMagenta", (139, 0, 139)),
    ("DarkOliveGreen", (85, 107, 47)),
    ("DarkOrange", (255, 140, 0)),
    ("DarkOrchid", (153, 50, 204)),
    ("DarkRed", (139, 0, 0)),
    ("DarkSalmon", (233, 150, 122)),
    ("DarkSeaGreen", (143, 188, 143)),
    ("DarkSlateBlue", (72, 61, 139)),
    ("DarkSlateGray", (47, 79, 79)),
    ("DarkTurquoise", (0, 206, 209)),
    ("DarkViolet", (148, 0, 211)),
    ("DeepPink", (255, 20, 147)),
    ("DeepSkyBlue", (0, 191, 255)),
    ("DodgerBlue", (30, 144, 255)),
    ("Firebrick", (178, 34, 34)),
    ("ForestGreen", (34, 139, 34)),
    ("Gainsboro", (220, 220, 220)),
    ("Gold", (255, 215, 0)),
    ("Goldenrod", (218, 165, 32)),
    ("GreenYellow", (173, 255, 47)),
    ("Honeydew", (240, 255, 240)),
    ("HotPink", (255, 105, 180)),
    ("IndianRed", (205, 92, 92)),
    ("Ivory", (255, 255, 240)),
    ("LemonChiffon", (255, 250, 205)),
    ("LightBlue", (173, 216, 230)),
    ("LightCoral", (240, 128, 128)),
    ("LightCyan", (224, 255, 255)),
    ("LightGoldenrod", (238, 221, 130)),
    ("LightGray", (211, 211, 211)),
    ("LightGreen", (144, 238, 144)),
    ("LightPink", (255, 182, 193)),
    ("LightSalmon", (255, 160, 122)),
    ("LightSeaGreen", (32, 178, 170)),
    ("LightSkyBlue", (135, 206, 250)),
    ("LightSlateGray", (119, 136, 153)),
    ("LightSteelBlue", (176, 196, 222)),
    ("LightYellow", (255, 255, 224)),
    ("LimeGreen", (50, 205, 50)),
    ("Linen", (250, 240, 230)),
    ("MediumAquamarine", (102, 205, 170)),
    ("MediumBlue", (0, 0, 205)),
    ("MediumOrchid", (186, 85, 211)),
    ("MediumPurple", (147, 112, 219)),
    ("MediumSeaGreen", (60, 179, 113)),
    ("MediumSlateBlue", (123, 104, 238)),
    ("MediumSpringGreen", (0, 250, 154)),
    ("MediumTurquoise", (72, 209, 204)),
    ("MediumVioletRed", (199, 21, 133)),
    ("MidnightBlue", (25, 25, 112)),
    ("MintCream", (245, 255, 250)),
    ("MistyRose", (255, 228, 225)),
    ("Moccasin", (255, 228, 181)),
    ("NavajoWhite", (255, 222, 173)),
    ("OldLace", (253, 245, 230)),
    ("OliveDrab", (107, 142, 35)),
    ("OrangeRed", (255, 69, 0)),
    ("Orchid", (218, 112, 214)),
    ("PaleGoldenrod", (238, 232, 170)),
    ("PaleGreen", (152, 251, 152)),
    ("PaleTurquoise", (175, 238, 238)),
    ("PaleVioletRed", (219, 112, 147)),
    ("PapayaWhip", (255, 239, 213)),
    ("PeachPuff", (255, 218, 185)),
    ("Peru", (205, 133, 63)),
    ("PowderBlue", (176, 224, 230)),
    ("RosyBrown", (188, 143, 143)),
    ("RoyalBlue", (65, 105, 225)),
    ("SaddleBrown", (139, 69, 19)),
    ("SeaGreen", (46, 139, 87)),
    ("Seashell", (255, 245, 238)),
    ("Sienna", (160, 82, 45)),
    ("SkyBlue", (135, 206, 235)),
    ("SlateBlue", (106, 90, 205)),
    ("SlateGray", (112, 128, 144)),
    ("Snow", (255, 250, 250)),
    ("SpringGreen", (0, 255, 127)),
    ("SteelBlue", (70, 130, 180)),
    ("Thistle", (216, 191, 216)),
    ("Turquoise", (64, 224, 208)),
    ("Vermilion", (227, 66, 52)),
    ("YellowGreen", (154, 205, 50)),
];
