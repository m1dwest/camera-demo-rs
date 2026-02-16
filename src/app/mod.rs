pub mod actions;
pub mod config;

use anyhow::{Result, ensure};

use eframe::egui;

use log::{debug, error, info};
use realsense_rust as rs;

use crate::core::{Camera, DevicesModel, RealSenseBackend};
use crate::ui::{device_mode_panel::DeviceModePanel, devices_combo_box::DevicesComboBox};
use actions::Action;
// use config::Config;

use crate::ui::status_bar::Message;

struct App {
    backend: Option<RealSenseBackend>,
    status: Message,

    devices_model: DevicesModel,
    devices_combo_box: DevicesComboBox,
    device_mode_panel: DeviceModePanel,

    camera: Option<Camera>,

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
            device_mode_panel: DeviceModePanel::new(false),

            camera: None,

            fatal_error,
        }
    }

    fn show_ui(&mut self, ctx: &egui::Context) -> Vec<Action> {
        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(12.0, 8.0);
        });
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
                ui.heading("Camera control");
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
                if let Err(e) = self.start_camera() {
                    self.status = Message::error(e.to_string());
                } else {
                    log::info!("Camera started successfully");
                    self.device_mode_panel.set_camera_active(true);
                }
            }
            Action::StopCamera => {
                info!("Action::StopCamera");
                // TODO:stop?
                self.camera = None;
                self.device_mode_panel.set_camera_active(false);
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

    fn start_camera(&mut self) -> Result<()> {
        let ctx = self.backend.as_ref().map(|b| b.context());
        let Some(ctx) = ctx else {
            anyhow::bail!("Unable to start camera. No valid realsense2 context found");
        };

        let serial = self
            .devices_model
            .selected_device()
            .and_then(|d| d.serial.as_ref());
        let Some(serial) = serial else {
            anyhow::bail!("Unable to start camera. Select the device first");
        };

        let stream = self.devices_model.selected_stream();
        let Some(stream) = stream else {
            anyhow::bail!("Unable to start camera. Select the stream first");
        };
        let mode = self.devices_model.selected_mode();
        let Some(mode) = mode else {
            anyhow::bail!("Unable to start camera. Select the resolution first");
        };

        let camera = Camera::new(serial, stream, mode, ctx);
        self.camera = match camera {
            Ok(c) => Some(c),
            Err(e) => {
                anyhow::bail!(e.to_string());
            }
        };
        Ok(())
    }

    // fn export_config(&self) -> Config {
    //     Config {
    //         devices_model: self.devices_model.export_config(),
    //     }
    // }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let actions = self.show_ui(ctx);
        self.execute_actions(actions);
    }

    // fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
    //     let cfg = self.export_config();
    //     confy::store("config.toml", None, cfg);
    // }
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
