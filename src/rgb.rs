use anyhow::bail;

#[derive(Copy, Clone)]
pub enum RequestType {
    SetColor = 0,
    SetEffect = 1,
}

#[derive(Copy, Clone, Default)]
pub struct RGBLedColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RGBLedColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
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
    pub request_type: RequestType,
    pub color: RGBLedColor,
}

impl RGBRequest {
    pub fn new(bytes: [u8; 4]) -> anyhow::Result<Self> {
        unsafe {
            if bytes[0] > 0 {
                bail!("пакет говна, иди в пизду")
            }

            Ok(Self {
                color: RGBLedColor::new(bytes[1], bytes[2], bytes[3]),
                request_type: std::mem::transmute::<u8, RequestType>(bytes[0]),
            })
        }
    }
}

impl Default for RGBRequest {
    fn default() -> Self {
        Self {
            request_type: RequestType::SetColor,
            color: RGBLedColor::default(),
        }
    }
}
