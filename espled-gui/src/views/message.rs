use eframe::egui::{self, vec2, Context, Label, RichText};

#[derive(PartialEq, PartialOrd)]
pub enum DialogType {
    Ok,
    Progress,
}

pub struct Message {
    content: String,
    dialog_type: DialogType,
}

impl Message {
    pub fn new<D: std::fmt::Display>(content: D, dialog_type: DialogType) -> Self {
        Self {
            content: content.to_string(),
            dialog_type,
        }
    }

    pub fn display(&mut self, ctx: &egui::Context) -> bool {
        let mut response = false;
        let icon = match self.dialog_type {
            DialogType::Ok => {
                "🚨"
            },
            DialogType::Progress => {
                "P"
            },
        };

        egui::Window::new("Message")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).size(50.0));
                    ui.label(self.content.clone());
                });
                if self.dialog_type == DialogType::Ok {
                    ui.separator();
                    if ui.button("OK").clicked() {
                        response = true;
                    }
                }
            });

        response
    }
}
