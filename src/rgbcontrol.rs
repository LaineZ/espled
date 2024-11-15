use crate::rgb::RGBLedColor;
use esp_idf_hal::ledc::LedcDriver;
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};

pub struct RgbControl<'a> {
    pwm_r: LedcDriver<'a>,
    pwm_g: LedcDriver<'a>,
    pwm_b: LedcDriver<'a>,
    nvs: EspNvsPartition<NvsDefault>,
    color: RGBLedColor,
}

impl<'a> RgbControl<'a> {
    pub fn new(
        pwm_r: LedcDriver<'a>,
        pwm_g: LedcDriver<'a>,
        pwm_b: LedcDriver<'a>,
        nvs: EspNvsPartition<NvsDefault>,
    ) -> Self {
        Self {
            pwm_r,
            pwm_g,
            pwm_b,
            nvs,
            color: RGBLedColor::default(),
        }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        let nvs_rgb = EspNvs::new(self.nvs.clone(), "rgb", true)?;

        if let Ok(color_u32) = nvs_rgb.get_u32("color") {
            self.color = RGBLedColor::new_from_u32(color_u32.unwrap_or(0xffffff));
        }

        self.set_color_pwm()?;
        Ok(())
    }

    pub fn set_color(&mut self, color: RGBLedColor) -> anyhow::Result<()> {
        self.color = color;
        self.set_color_pwm()?;
        let nvs_rgb = EspNvs::new(self.nvs.clone(), "rgb", true).unwrap();
        nvs_rgb.set_u32("color", self.color.to_u32())?;
        Ok(())
    }

    fn set_color_pwm(&mut self) -> anyhow::Result<()> {
        self.pwm_r.set_duty(self.color.red as u32)?;
        self.pwm_g.set_duty(self.color.green as u32)?;
        self.pwm_b.set_duty(self.color.blue as u32)?;
        Ok(())
    }
}
