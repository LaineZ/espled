use crate::rgb::RGBLedColor;
use esp_idf_hal::ledc::LedcDriver;
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};

const RED_MULTIPLIER: u32 = 255;
const GREEN_MULTIPLIER: u32 = 255;
const BLUE_MULTIPLIER: u32 = 255;
const MULTIPLIER_DIVISOR: u32 = 255;

pub struct RgbControl {
    pwm_r: LedcDriver<'static>,
    pwm_g: LedcDriver<'static>,
    pwm_b: LedcDriver<'static>,
    nvs: EspNvsPartition<NvsDefault>,
    color: RGBLedColor,
}

impl RgbControl {
    pub fn new(
        pwm_r: LedcDriver<'static>,
        pwm_g: LedcDriver<'static>,
        pwm_b: LedcDriver<'static>,
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

    pub fn get_color(&self) -> RGBLedColor {
        self.color
    }

    pub fn set_color(&mut self, color: RGBLedColor) -> anyhow::Result<()> {
        self.color = color;
        self.set_color_pwm()?;
        let nvs_rgb = EspNvs::new(self.nvs.clone(), "rgb", true).unwrap();
        nvs_rgb.set_u32("color", self.color.to_u32())?;
        Ok(())
    }

    fn set_color_pwm(&mut self) -> anyhow::Result<()> {
        let max_duty = self.pwm_r.get_max_duty();
        self.pwm_r.set_duty(
            self.color.red as u32 * RED_MULTIPLIER / MULTIPLIER_DIVISOR * max_duty / 255,
        )?;
        self.pwm_g.set_duty(
            self.color.green as u32 * GREEN_MULTIPLIER / MULTIPLIER_DIVISOR * max_duty / 255,
        )?;
        self.pwm_b.set_duty(
            self.color.blue as u32 * BLUE_MULTIPLIER / MULTIPLIER_DIVISOR * max_duty / 255,
        )?;
        Ok(())
    }
}
