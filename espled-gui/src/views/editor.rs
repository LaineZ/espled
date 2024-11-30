use eframe::egui::{self, widgets};

use super::View;

pub struct EditorView {
    color: [f32; 3],
}

impl Default for EditorView {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
        }
    }
}

impl EditorView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_color(&self) -> u32 {
        let r = (self.color[0].clamp(0.0, 1.0) * 255.0) as u32;
        let g = (self.color[1].clamp(0.0, 1.0) * 255.0) as u32;
        let b = (self.color[2].clamp(0.0, 1.0) * 255.0) as u32;

        (r << 16) | (g << 8) | b
    }
}

impl View for EditorView {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            ui.horizontal(|ui| {
                ui.label("Color");
                widgets::color_picker::color_edit_button_rgb(ui, &mut self.color);
            });
        });
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
