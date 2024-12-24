use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct RGBLedColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RGBLedColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
    /// H - [0.0, 360.0], S - [0.0, 1.0], V - [0.0, 1.0]
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let c = v * s; // chroma
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = match h {
            0.0..=60.0 => (c, x, 0.0),
            60.0..=120.0 => (x, c, 0.0),
            120.0..=180.0 => (0.0, c, x),
            180.0..=240.0 => (0.0, x, c),
            240.0..=300.0 => (x, 0.0, c),
            300.0..=360.0 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };

        let red = ((r1 + m) * 255.0).round() as u8;
        let green = ((g1 + m) * 255.0).round() as u8;
        let blue = ((b1 + m) * 255.0).round() as u8;

        Self { red, green, blue }
    }

    pub fn new_from_u32(color: u32) -> Self {
        Self {
            red: ((color >> 16) & 0xFF) as u8,
            green: ((color >> 8) & 0xFF) as u8,
            blue: (color & 0xFF) as u8,
        }
    }

    pub fn to_u32(&self) -> u32 {
        let red = (self.red as u32) << 16;
        let green = (self.green as u32) << 8;
        let blue = self.blue as u32;

        red | green | blue
    }
}

#[derive(Copy, Clone)]
pub struct RGBRequest {
    pub color: RGBLedColor,
}

impl RGBRequest {
    pub fn new(bytes: [u8; 3]) -> anyhow::Result<Self> {
        Ok(Self {
            color: RGBLedColor::new(bytes[0], bytes[1], bytes[2]),
        })
    }
}

impl Default for RGBRequest {
    fn default() -> Self {
        Self {
            color: RGBLedColor::default(),
        }
    }
}
