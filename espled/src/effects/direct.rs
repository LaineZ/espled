use super::{Effect, ParameterTypes};
use crate::rgb::RGBLedColor;
use std::collections::HashMap;

#[derive(Default)]
pub struct Direct {
    parameters: HashMap<String, super::ParameterTypes>,
}

impl Direct {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::from([(
                String::from("color"),
                ParameterTypes::Color(RGBLedColor::default()),
            )]),
        }
    }

    fn get_color(&self) -> RGBLedColor {
        match self.parameters.get("color") {
            Some(ParameterTypes::Color(color)) => *color,
            _ => RGBLedColor::default(),
        }
    }
}

impl Effect for Direct {
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
        "Direct"
    }

    fn update(&mut self, _delta_time: f32) {}

    fn render(&self) -> RGBLedColor {
        self.get_color()
    }
}
