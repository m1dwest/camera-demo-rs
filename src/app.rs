use anyhow::{Result, ensure};
use eframe::egui;

use log::{debug, error, info};
use realsense_rust as rs;

use crate::actions::Action;
use crate::core::Camera;
use crate::core::{Device, DevicesModel, RealSenseBackend};
use crate::ui::{device_mode_panel::DeviceModePanel, devices_combo_box::DevicesComboBox};

use crate::ui::status_bar::Message;

struct App {
    backend: Option<RealSenseBackend>,
    status: Message,

    devices_model: DevicesModel,
    devices_combo_box: DevicesComboBox,
    device_mode_panel: DeviceModePanel,

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
        let devices_model = DevicesModel::from_devices(devices, None);

        let devices_combo_box = DevicesComboBox::new("Available devices");

        Self {
            backend,
            status: Message::none(),

            devices_model,
            devices_combo_box,
            device_mode_panel: DeviceModePanel::new(),
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
            let combo_box_actions = self.devices_combo_box.show(ui, &self.devices_model);
            actions.extend(combo_box_actions);
        });

        egui::SidePanel::left("control_panel")
            .exact_width(300.0)
            .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::same(8)))
            .show(ctx, |ui| {
                ui.heading("Control panel");
                let device_mode_actions = self.device_mode_panel.show(ui, &self.devices_model);
                actions.extend(device_mode_actions);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            //
        });

        actions
    }

    fn execute_actions(&mut self, actions: Vec<Action>) {
        if actions.is_empty() {
            return;
        }

        self.status = Message::none();

        actions.into_iter().for_each(|action| match action {
            Action::RefreshDevices => {
                info!("Action::RefreshDevices executed");
                self.refresh_devices();
            }
            Action::StartCamera => {
                info!("Action::StartCamera");
            }
            Action::StopCamera => {
                info!("Action::StopCamera");
            }
            Action::ChangeCamera { serial } => {
                info!("Action::ChangeCamera {}", serial);
                self.change_camera(serial);
            }
            Action::SelectSensor { sensor } => {
                info!("Action::SelectSensor {}", sensor);
                self.select_sensor(sensor);
            }
            Action::SelectStream { stream } => {
                info!("Action::SelectStream {}", stream);
                self.select_stream(stream);
            }
            Action::SelectMode { mode } => {
                info!("Action::SelectMode {}", mode);
                self.select_mode(mode);
            }
            Action::None => {}
        });
    }

    fn refresh_devices(&mut self) {
        let devices = self
            .backend
            .as_ref()
            .expect("Program is running with empty backend")
            .devices();
        self.devices_model =
            DevicesModel::from_devices(devices, Some(std::mem::take(&mut self.devices_model)));
    }

    fn change_camera(&mut self, serial: String) {
        let ok = self.devices_model.select_device(serial);
        if let Err(e) = ok {
            self.status = Message::error(e.to_string());
        }
    }

    fn select_sensor(&mut self, sensor: String) {
        let ok = self.devices_model.select_sensor(sensor);
        if let Err(e) = ok {
            self.status = Message::error(e.to_string());
        }
    }

    fn select_stream(&mut self, stream: realsense_rust::kind::Rs2StreamKind) {
        let ok = self.devices_model.select_stream(stream);
        if let Err(e) = ok {
            self.status = Message::error(e.to_string());
        }
    }

    fn select_mode(&mut self, mode: crate::core::Mode) {
        let ok = self.devices_model.select_mode(mode);
        if let Err(e) = ok {
            self.status = Message::error(e.to_string());
        }
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
