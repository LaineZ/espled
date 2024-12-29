use control_thread::{ChannelStatus, ControlChannel, Controller};
use eframe::egui::{self, menu, vec2};
use egui_extras::{Column, TableBuilder};
use views::{connection::ConnectionView, editor::EditorView, ToggledViewManager, View};

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
    message_dialog: views::message::Message,
    control_thread: ControlChannel,
    selected_controller: Option<Controller>,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            connection_view: ToggledViewManager::new(Box::new(ConnectionView::default())),
            editor_view: EditorView::default(),
            message_dialog: views::message::Message::default(),
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
                    TableBuilder::new(ui)
                        .column(Column::auto().resizable(false))
                        .body(|mut body| {
                            for controller in self.control_thread.get_controllers().iter() {
                                body.row(16.0, |mut row| {
                                    row.col(|ui| {
                                        if ui.button(&controller.name).clicked() {
                                            self.selected_controller = Some(controller.clone());
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
                            body.row(14.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button("Discover serial").clicked() {
                                        self.control_thread.discover_controllers();
                                    }
                                });
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
            if let Some(controller) = self.selected_controller.clone() {
                self.editor_view.ui(ui);
                controller.apply_color(self.editor_view.get_color());
            } else {
                ui.heading("Please select MCLU connection from list");
            }
        });

        egui::TopBottomPanel::bottom("my_bottom_panel").show(ctx, |ui| {
            ui.label(format!("{}", self.control_thread.status()))
        });

        match self.control_thread.status() {
            ChannelStatus::ProbingControllers(controller) => {
                self.message_dialog.show(
                    format!("Probing controller on port {controller}. Please wait..."),
                    views::message::DialogType::Progress,
                );
            }
            ChannelStatus::Done => {
                self.message_dialog.hide();
            }
        }

        self.message_dialog.display(ctx);
    }
}
