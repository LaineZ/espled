use protocol::RGBLedColor;

use super::{Effect, ParameterTypes};
use std::collections::HashMap;

pub struct Decay {
    parameters: HashMap<String, super::ParameterTypes>,
    color: f32,
}

impl Decay {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::from([
                (String::from("speed"), ParameterTypes::Float(1.0)),
            ]),
            color: 0.0,
        }
    }
}

impl Effect for Decay {
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
        "Decay"
    }

    fn update(&mut self, delta_time: f32) {
        let speed = self.parameters.get("speed").unwrap().as_f32().unwrap();
        
        if self.color < 1.0 {
            self.color += speed * delta_time;
        } else {
            self.color = 0.0;
        }
    }

    fn render(&self) -> RGBLedColor {
        let component = (self.color * 255.0) as u8;
        RGBLedColor::new(component, component, component)
    }
}
