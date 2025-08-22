use serde::{Deserialize, Serialize};


pub const DEFAULT_GAMMA_COEFICIENT: f32 = 2.2;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ParameterTypes {
    Color(RGBLedColor),
    Float(f32),
}

impl ParameterTypes {
    pub fn as_f32(self) -> Option<f32> {
        match self {
            ParameterTypes::Color(_) => {
                None
            },
            ParameterTypes::Float(value) => Some(value),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetEffects,
    GetEffect,
    GetParameters,
    GetName,
    SetEffect(usize),
    SetOption(String, ParameterTypes),
}


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

    pub fn gamma_correct(&mut self, coeficient: f32) {
        self.red = ((self.red as f32 / 255.0).powf(coeficient) * 255.0).round() as u8;
        self.green = ((self.green as f32 / 255.0).powf(coeficient) * 255.0).round() as u8;
        self.blue = ((self.blue as f32 / 255.0).powf(coeficient) * 255.0).round() as u8;
    }

    pub fn to_u32(&self) -> u32 {
        let red = (self.red as u32) << 16;
        let green = (self.green as u32) << 8;
        let blue = self.blue as u32;

        red | green | blue
    }
}


impl Into<[f32; 3]> for RGBLedColor {
    fn into(self) -> [f32; 3] {
        [
            self.red as f32 / 255.0,
            self.green as f32 / 255.0,
            self.blue as f32 / 255.0,
        ]
    }
}

impl From<[f32; 3]> for RGBLedColor {
    fn from(arr: [f32; 3]) -> Self {
        RGBLedColor {
            red: (arr[0] * 255.0).round() as u8,
            green: (arr[1] * 255.0).round() as u8,
            blue: (arr[2] * 255.0).round() as u8,
        }
    }
}

impl PartialEq for RGBLedColor {
    fn eq(&self, other: &Self) -> bool {
        self.to_u32() == other.to_u32()
    }
}

impl Eq for RGBLedColor {}
