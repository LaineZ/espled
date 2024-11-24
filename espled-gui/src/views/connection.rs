use eframe::egui;
use std::net::{AddrParseError, Ipv4Addr};

use super::View;

pub struct ConnectionView {
    ip_address: String,
    port: f32,
    pub connect_button_clicked: bool,
}

impl Default for ConnectionView {
    fn default() -> Self {
        Self {
            port: 80.0,
            ip_address: String::default(),
            connect_button_clicked: false,
        }
    }
}

impl ConnectionView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_ip_address(&self) -> Result<Ipv4Addr, AddrParseError> {
        self.ip_address.parse()
    }

    pub fn get_port(&self) -> u16 {
        self.port.clamp(0.0, 65536.0) as u16
    }
}

impl View for ConnectionView {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            ui.label("IP Address:");
            ui.text_edit_singleline(&mut self.ip_address);
            ui.label("Port:");
            ui.add(
                egui::DragValue::new(&mut self.port)
                    .range(1.0..=65536.0)
                    .speed(1.0)
                    .fixed_decimals(0),
            );
            if let Err(message) = self.get_ip_address() {
                ui.small(format!("{}", message));
            } else {
                ui.small("");
            }
            ui.add_enabled_ui(self.get_ip_address().is_ok(), |ui| {
                self.connect_button_clicked = ui
                    .add(egui::Button::new("Connect").fill(egui::Color32::from_rgb(0, 0, 90)))
                    .clicked();
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
