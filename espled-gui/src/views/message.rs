use eframe::egui::{self, vec2};

#[derive(PartialEq, PartialOrd)]
pub enum DialogType {
    Ok,
    Progress,
}

pub struct Message {
    content: String,
    dialog_type: DialogType,
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

impl Message {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            dialog_type: DialogType::Ok,
        }
    }

    pub fn show<D: std::fmt::Display>(&mut self, message: D, dialog_type: DialogType) {
        self.content = message.to_string();
        self.dialog_type = dialog_type;
    }

    pub fn display(&mut self, ctx: &egui::Context) {
        if self.content.is_empty() {
            return;
        }

        egui::Window::new("Message")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.label(self.content.clone());
                if self.dialog_type == DialogType::Ok {
                    ui.separator();
                    if ui.button("OK").clicked() {
                        self.content = String::new();
                    }
                }
            });
    }
}
