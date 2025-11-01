use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn from_float(r: f32, g: f32, b: f32) -> Self {
        Color {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    pub fn to_hex(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::new(
            (self.r as f32 * (1.0 - t) + other.r as f32 * t) as u8,
            (self.g as f32 * (1.0 - t) + other.g as f32 * t) as u8,
            (self.b as f32 * (1.0 - t) + other.b as f32 * t) as u8,
        )
    }

    pub fn mul(&self, factor: f32) -> Color {
        Color::from_float(
            self.r as f32 / 255.0 * factor,
            self.g as f32 / 255.0 * factor,
            self.b as f32 / 255.0 * factor,
        )
    }

    pub fn add(&self, other: &Color) -> Color {
        Color::from_float(
            (self.r as f32 + other.r as f32) / 255.0,
            (self.g as f32 + other.g as f32) / 255.0,
            (self.b as f32 + other.b as f32) / 255.0,
        )
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color(r: {}, g: {}, b: {})", self.r, self.g, self.b)
    }
}
