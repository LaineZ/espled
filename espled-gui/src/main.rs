use control_thread::{ChannelStatus, ControlChannel};
use eframe::egui::{self, vec2, widgets};
use egui_extras::{Column, TableBuilder};
use views::{connection::ConnectionView, ToggledViewManager, View};

pub mod control_thread;
pub mod views;

fn color_to_u32(color: [f32; 3]) -> u32 {
    let r = (color[0].clamp(0.0, 1.0) * 255.0) as u32;
    let g = (color[1].clamp(0.0, 1.0) * 255.0) as u32;
    let b = (color[2].clamp(0.0, 1.0) * 255.0) as u32;

    (r << 16) | (g << 8) | b
}

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
    selected_controller_index: Option<usize>,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            connection_view: ToggledViewManager::new(Box::new(ConnectionView::default())),
            message_dialog: views::message::Message::default(),
            control_thread: ControlChannel::new(),
            selected_controller_index: Some(0),
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
                        for (index, controller) in
                            self.control_thread.get_controllers().iter().enumerate()
                        {
                            body.row(16.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button(&controller.name).clicked() {
                                        self.selected_controller_index = Some(index);
                                    }
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
            if let Some(index) = self.selected_controller_index {
                let mut rgb = [0.0, 0.0, 0.0];
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    widgets::color_picker::color_edit_button_rgb(ui, &mut rgb);
                });
            } else {
                ui.heading("Please select MCLU connection from list");
            }
        });

        egui::TopBottomPanel::bottom("my_bottom_panel").show(ctx, |ui| {
            ui.label(format!("{}", self.control_thread.status()))
        });
        self.message_dialog.display(ctx);
        self.process(ctx);
    }
}
