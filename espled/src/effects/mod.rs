use std::collections::HashMap;

use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use protocol::{ParameterTypes, RGBLedColor};

pub mod direct;
pub mod huerotate;

pub trait Effect {
    fn get_parameters(&self) -> HashMap<String, ParameterTypes>;
    fn set_parameter(&mut self, parameter_name: &str, value: ParameterTypes) -> bool;
    fn name(&self) -> &str;
    fn init(&mut self, nvs_partition: EspNvsPartition<NvsDefault>) {
        let nvs = EspNvs::new(nvs_partition.clone(), self.name(), true).unwrap();

        for (key, value) in self.get_parameters() {
            match value {
                ParameterTypes::Color(_) => {
                    if let Ok(color_u32) = nvs.get_u32(&key) {
                        self.set_parameter(
                            &key,
                            ParameterTypes::Color(RGBLedColor::new_from_u32(
                                color_u32.unwrap_or(0xffffff),
                            )),
                        );
                    }
                }
                ParameterTypes::Float(_) => {
                    if let Ok(float_bits) = nvs.get_u32(&key) {
                        self.set_parameter(
                            &key,
                            ParameterTypes::Float(f32::from_bits(float_bits.unwrap_or(0))),
                        );
                    }
                }
            }
        }
    }
    fn save(&mut self, nvs_partition: EspNvsPartition<NvsDefault>) {
        let parameters = self.get_parameters();
        let nvs = EspNvs::new(nvs_partition.clone(), self.name(), true).unwrap();
        for (key, value) in parameters {
            match value {
                ParameterTypes::Color(rgbled_color) => {
                    nvs.set_u32(&key, rgbled_color.to_u32()).unwrap();
                }
                ParameterTypes::Float(value) => {
                    nvs.set_u32(&key, value.to_bits()).unwrap();
                }
            }
        }
    }
    fn update(&mut self, delta_time: f32);
    fn render(&self) -> RGBLedColor;
}
