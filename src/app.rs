use anyhow::{Result, ensure};
use eframe::egui;

use log::{debug, error, info};
use realsense_rust as rs;

use crate::actions::Action;
use crate::core::Camera;
use crate::core::RealSenseBackend;
use crate::ui::devices_combo_box::DevicesComboBox;
use crate::ui::status_bar::Message;

struct App {
    backend: Option<RealSenseBackend>,
    status: Message,
    devices_combo_box: DevicesComboBox,

    fatal_error: Option<String>,
}

impl App {
    fn new() -> Self {
        let (backend, fatal_error) = match RealSenseBackend::new() {
            Ok(value) => (Some(value), None),
            Err(e) => (None, Some(format!("{:#}", e))),
        };

        let devices = backend
            .as_ref()
            .map_or(Vec::new(), |backend| backend.devices());
        let devices_combo_box = DevicesComboBox::new("Available devices", devices);

        // let selected_sn = devices_combo_box.selected_sn();
        // let selected_mode = devices_combo_box.selected_mode();

        Self {
            backend,
            status: Message::none(),
            devices_combo_box,
            fatal_error,
        }
    }

    fn show_ui(&mut self, ctx: &egui::Context) -> Vec<Action> {
        let mut actions = Vec::new();

        if let Some(error) = &self.fatal_error {
            crate::ui::fatal_popup::show(ctx, error);
            return actions;
        }

        crate::ui::status_bar::show(ctx, &self.status);

        egui::TopBottomPanel::top("device_select_panel").show(ctx, |ui| {
            let combo_box_actions = self.devices_combo_box.show(ui);
            actions.extend(combo_box_actions);
        });

        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("Control panel");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            //
        });

        actions
    }

    fn execute_actions(&mut self, actions: Vec<Action>) {
        actions.iter().for_each(|action| match action {
            Action::RefreshDeviceList => {
                let devices = self
                    .backend
                    .as_ref()
                    .expect("Program is running with empty backend")
                    .devices();
                self.devices_combo_box.refresh_device_list(devices);

                info!("Action::RefreshDeviceList executed");
            }
            Action::DisableCamera => {
                info!("Action::DisableCamera");
            }
            Action::ChangeCamera { serial } => {
                info!("Action::ChangeCamera {}", serial);
            }
            Action::None => {}
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let actions = self.show_ui(ctx);
        self.execute_actions(actions);
    }
}

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Camera Demo",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}
