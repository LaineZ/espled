use control_thread::{ChannelStatus, ControlChannel};
use eframe::egui::{self, vec2};
use egui_extras::{Column, TableBuilder};
use views::{connection::ConnectionView, ToggledViewManager, View};

pub mod control_thread;
pub mod views;

fn main() {
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
    message_dialog: views::message::Message,
    control_thread: ControlChannel,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            connection_view: ToggledViewManager::new(Box::new(ConnectionView::default())),
            message_dialog: views::message::Message::default(),
            control_thread: ControlChannel::new(),
        }
    }

    fn process(&mut self, ctx: &egui::Context) {
        let controllers = self.control_thread.get_controllers();

        if controllers.len() == 0 {
            self.control_thread.discover_controllers();
        }

        let connect_view = self
            .connection_view
            .as_original::<ConnectionView>()
            .unwrap();
        if connect_view.connect_button_clicked {
            self.message_dialog.show(
                "Establishing connection...",
                views::message::DialogType::Progress,
            );
            self.connection_view.enabled = false;
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::new(egui::panel::Side::Left, "side_panel")
            .resizable(true)
            .min_width(100.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.add_space(5.0);
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(false))
                    .body(|mut body| {
                        for controller in self.control_thread.get_controllers() {
                            body.row(16.0, |mut row| {
                                row.col(|ui| {
                                    ui.button(controller.name);
                                });
                            });
                        }
                        body.row(14.0, |mut row| {
                            row.col(|ui| {
                                if ui.button("Add remote").clicked() {
                                    self.connection_view.enabled = true;
                                }
                            });
                        });
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
            ui.heading("Please select MCLU connection from list");
        });

        egui::TopBottomPanel::bottom("my_bottom_panel").show(ctx, |ui| {
            ui.label(format!("{}", self.control_thread.status()))
        });
        self.message_dialog.display(ctx);
        self.process(ctx);
    }
}
