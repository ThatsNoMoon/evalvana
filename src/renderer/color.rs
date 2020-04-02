use zerocopy::{
    AsBytes,
    FromBytes,
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, AsBytes, FromBytes)]
pub struct Color {
    pub rgb: [f32; 3],
}

use std::fmt;

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgb({}, {}, {})", self.rgb[0], self.rgb[1], self.rgb[2])
    }
}

impl fmt::LowerHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b] = self.rgb;
        let (r, g, b) = ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
        write!(f, "#{:x}{:x}{:x}", r, g, b)
    }
}

impl fmt::UpperHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b] = self.rgb;
        let (r, g, b) = ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
        write!(f, "#{:X}{:X}{:X}", r, g, b)
    }
}

impl Color {
    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Color {
        Color {
            rgb: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0],
        }
    }

    pub fn from_rgb_f32(r: f32, g: f32, b: f32) -> Color {
        Color {
            rgb: [r, g, b],
        }
    }

    pub fn from_rgb_u32(rgb: u32) -> Color {
        let (r, g, b) = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF);
        let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        Color {
            rgb: [r, g, b],
        }
    }

    pub fn to_rgba(&self) -> [f32; 4] {
        let [r, g, b] = self.rgb;
        [r, g, b, 1.0]
    }

    pub fn to_wgpu(&self) -> wgpu::Color {
        let [r, g, b] = self.rgb;
        wgpu::Color {
            r: r as f64,
            g: g as f64,
            b: b as f64,
            a: 1.0,
        }
    }
}
