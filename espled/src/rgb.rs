use protocol::RGBLedColor;

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
