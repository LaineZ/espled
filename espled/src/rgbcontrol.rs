use std::{collections::HashMap, time::Instant};

use crate::effects::{self, Effect};
use esp_idf_hal::ledc::LedcDriver;
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use protocol::{ParameterTypes, RGBLedColor};

pub struct RgbControl {
    pwm_r: LedcDriver<'static>,
    pwm_g: LedcDriver<'static>,
    pwm_b: LedcDriver<'static>,
    nvs: EspNvsPartition<NvsDefault>,
    effects: Vec<Box<dyn Effect>>,
    selected_effect_index: usize,
    dt: Instant,
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
            effects: vec![
                Box::new(effects::direct::Direct::new()),
                Box::new(effects::huerotate::HueRotate::new()),
            ],
            selected_effect_index: 0,
            dt: Instant::now(),
        }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        let nvs_handle_settings = EspNvs::new(self.nvs.clone(), "settings", true)?;
        self.selected_effect_index =
            if let Ok(effect_index) = nvs_handle_settings.get_u8("effect_index") {
                effect_index.unwrap_or_default() as usize
            } else {
                0
            };

        Ok(())
    }

    pub fn set_effect(&mut self, index: usize) -> bool {
        if index > self.effects.len() - 1 {
            return false;
        }
        self.selected_effect_index = index;
        true
    }

    pub fn get_effect_name(&self) -> &str {
        self.effects[self.selected_effect_index].name()
    }

    pub fn get_effect_options(&self) -> HashMap<String, ParameterTypes> {
        self.effects[self.selected_effect_index].get_parameters()
    }

    pub fn set_effect_parameter(&mut self, name: &str, value: ParameterTypes) {
        self.effects[self.selected_effect_index].set_parameter(name, value);
        self.effects[self.selected_effect_index].save(self.nvs.clone());
    }

    pub fn get_effects_name(&self) -> Vec<&str> {
        self.effects.iter().map(|x| x.name()).collect()
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        self.effects[self.selected_effect_index].update(self.dt.elapsed().as_secs_f32());
        self.dt = Instant::now();
        let color = self.effects[self.selected_effect_index].render();
        self.set_color_pwm(color)
    }

    fn set_color_pwm(&mut self, color: RGBLedColor) -> anyhow::Result<()> {
        let max_duty = self.pwm_r.get_max_duty();
        self.pwm_r.set_duty(color.red as u32 * max_duty / 255)?;
        self.pwm_g.set_duty(color.green as u32 * max_duty / 255)?;
        self.pwm_b.set_duty(color.blue as u32 * max_duty / 255)?;
        Ok(())
    }
}
