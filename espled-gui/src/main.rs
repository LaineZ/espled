use crate::egui::FontFamily::Proportional;
use crate::egui::TextStyle::Heading;
use crate::egui::TextStyle::Name;
use std::collections::BTreeMap;

use control_thread::{ChannelStatus, ControlChannel, Controller};
use eframe::egui::{self, menu, vec2, FontId};
use egui_extras::{Column, TableBuilder};
use views::{
    connection::ConnectionView, editor::EditorView, message::Message, ToggledViewManager, View,
};

pub mod control_thread;
pub mod views;

fn main() {
    env_logger::init();
    log::info!("Started");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "egulenta",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
    .unwrap();
}

struct MyEguiApp {
    connection_view: ToggledViewManager,
    editor_view: EditorView,
    control_thread: ControlChannel,
    selected_controller: Option<Controller>,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            connection_view: ToggledViewManager::new(Box::new(ConnectionView::default())),
            editor_view: EditorView::default(),
            control_thread: ControlChannel::new(),
            selected_controller: None,
        }
    }

    fn process(&mut self, ctx: &egui::Context) {
        let connect_view = self
            .connection_view
            .as_original::<ConnectionView>()
            .unwrap();
        if connect_view.connect_button_clicked {
            // TODO
            self.connection_view.enabled = false;
        }
    }
}

impl eframe::App for MyEguiApp {
    fn raw_input_hook(&mut self, ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        self.process(ctx);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(egui::panel::Side::Left, "side_panel")
            .resizable(true)
            .min_width(100.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                menu::bar(ui, |ui| {
                    ui.menu_button("Tools", |ui| if ui.button("Serial monitor").clicked() {});
                });

                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    ui.label("Controllers:");
                    for controller in self.control_thread.get_controllers().iter() {
                        if ui.button(&controller.name).clicked() {
                            self.selected_controller = Some(controller.clone());
                            log::info!("Initialized controller: {:?}", controller);
                            self.editor_view = EditorView::new(controller);
                        }
                    }
                    if ui.button("Discover serial").clicked() {
                        self.control_thread.discover_controllers();
                    }
                });
            });
        egui::Window::new("Connection")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .default_width(200.0)
            .open(&mut self.connection_view.enabled)
            .show(ctx, |ui| {
                self.connection_view.view.ui(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(mut controller) = self.selected_controller.clone() {
                self.editor_view.ui(ui);
                if self.editor_view.changed_effect {
                    controller.set_effect(&self.editor_view.selected_effect);
                    self.editor_view.options = controller.options.clone();
                    self.editor_view.changed_effect = false;
                }

                if self.editor_view.changed_option {
                    controller.options = self.editor_view.options.clone();
                    controller.set_options();
                    self.editor_view.changed_option = false;
                }
            } else {
                ui.heading("Please select MCLU connection from list");
            }
        });

        egui::TopBottomPanel::bottom("my_bottom_panel").show(ctx, |ui| {
            ui.label(format!("{}", self.control_thread.status()))
        });

        match self.control_thread.status() {
            ChannelStatus::ProbingControllers(_) => {
                Message::new(
                    "Probing controllers, please wait",
                    views::message::DialogType::Progress,
                )
                .display(ctx);
            }
            ChannelStatus::NoControllers => {
                if Message::new("No controllers found", views::message::DialogType::Ok).display(ctx)
                {
                    log::trace!("acknown");
                    self.control_thread.acknown_status();
                }
            }
            _ => {}
        }
    }
}
