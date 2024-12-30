use std::{collections::HashMap, env::consts};

use eframe::egui::{self, widgets};
use protocol::{ParameterTypes, RGBLedColor};

use crate::control_thread::Controller;

use super::View;
#[derive(Default)]
pub struct EditorView {
    pub options: HashMap<String, ParameterTypes>,
    pub effects: Vec<String>,
    pub selected_effect: String,
    pub changed_option: bool,
    pub changed_effect: bool,
}


impl EditorView {
    pub fn new(controller: &Controller) -> Self {
        Self {
            options: controller.options.clone(),
            effects: controller.effect_list.clone(),
            changed_option: false,
            changed_effect: false,
            selected_effect: controller.get_effect(),
        }
    }
}

impl View for EditorView {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            ui.horizontal(|ui| {
                ui.label("Effect");
                egui::ComboBox::from_id_salt(41951919)
                .selected_text(format!("{:?}", self.selected_effect))
                .show_ui(ui, |ui| {

                    let mut new_effect = self.selected_effect.clone();
                    for effect in self.effects.iter() {
                        ui.selectable_value(&mut new_effect, effect.clone(), effect);
                    }
                    if new_effect != self.selected_effect {
                        self.selected_effect = new_effect;
                        self.changed_effect = true;
                    }
                }
                );
            });
            for (key, value) in self.options.iter_mut() {
                ui.horizontal(|ui| {
                    ui.label(key);
                    match value {
                        ParameterTypes::Color(rgbled_color) => {
                            let mut color_array: [f32; 3] = (*rgbled_color).into();
                            widgets::color_picker::color_edit_button_rgb(ui, &mut color_array);
                            let new_color = RGBLedColor::from(color_array);
                            if new_color != *rgbled_color {
                                *rgbled_color = new_color;
                                //self.changed_option = true;
                            }
                        },
                        ParameterTypes::Float(value) => {
                            let new_value = value.clone();
                            if ui.add(egui::Slider::new(value, 0.0..=100.0)).changed() {
                                //self.changed_option = true;
                            }
                        },
                    }
                });
            }

            if ui.button("Apply options").clicked() {
                self.changed_option = true;
            };
        });
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
