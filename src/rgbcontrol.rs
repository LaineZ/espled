use esp_idf_hal::ledc::LedcDriver;
use crate::rgb::RGBLedColor;

pub struct RgbControl<'a> {
   pwm_r: LedcDriver<'a>,
   pwm_g: LedcDriver<'a>,
   pwm_b: LedcDriver<'a>,
}

impl <'a>RgbControl<'a> {
    pub fn new(pwm_r: LedcDriver<'a>, pwm_g: LedcDriver<'a>, pwm_b: LedcDriver<'a>) -> Self {
        Self {
            pwm_r, pwm_g, pwm_b
        }
    }

    pub fn set_color(&mut self, color: RGBLedColor) -> anyhow::Result<()> {
        self.pwm_r.set_duty(color.red as u32)?;
        self.pwm_g.set_duty(color.green as u32)?;
        self.pwm_b.set_duty(color.blue as u32)?;
        Ok(())
    }
}
