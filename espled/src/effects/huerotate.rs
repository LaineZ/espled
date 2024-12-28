use protocol::RGBLedColor;

use super::{Effect, ParameterTypes};
use std::collections::HashMap;

pub struct HueRotate {
    parameters: HashMap<String, super::ParameterTypes>,
    hue: f32,
}

impl HueRotate {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::from([
                (String::from("saturation"), ParameterTypes::Float(1.0)),
                (String::from("value"), ParameterTypes::Float(1.0)),
                (String::from("speed"), ParameterTypes::Float(1.0)),
            ]),
            hue: 0.0,
        }
    }
}

impl Effect for HueRotate {
    fn get_parameters(&self) -> HashMap<String, ParameterTypes> {
        self.parameters.clone()
    }

    fn set_parameter(&mut self, parameter: &str, value: ParameterTypes) -> bool {
        if self.parameters.get(parameter).is_none() {
            return false;
        }

        self.parameters.insert(parameter.to_string(), value);
        true
    }

    fn name(&self) -> &str {
        "Hue Rotate"
    }

    fn update(&mut self, delta_time: f32) {
        let speed = self.parameters.get("speed").unwrap().as_f32().unwrap();
        if self.hue >= 360.0 {
            self.hue = 0.0;
        } else {
            self.hue += speed * delta_time;
        }
    }

    fn render(&self) -> RGBLedColor {
        let saturation = self.parameters.get("speed").unwrap().as_f32().unwrap();
        let value = self.parameters.get("value").unwrap().as_f32().unwrap();
        RGBLedColor::from_hsv(self.hue, saturation, value)
    }
}
