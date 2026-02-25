pub mod actions;
pub mod config;

use anyhow::{Result, ensure};

use crossbeam_channel::{Receiver, Sender, select};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use std::{
    thread,
    time::{Duration, Instant},
};

use eframe::egui;

use log::{debug, error, info};
use realsense_rust as rs;

use crate::core::{Camera, DevicesModel, Frame, PixelFormat, RealSenseBackend};
use crate::ui::{CameraView, DeviceModePanel, DevicesComboBox};
use actions::{Action, CameraAction};
// use config::Config;

use crate::app::config::Config;
use crate::ui::status_bar::Message;

struct CameraStatus {
    is_active: Arc<AtomicBool>,
    width: usize,
    height: usize,
}

impl CameraStatus {
    fn stopped() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            width: 0,
            height: 0,
        }
    }

    fn started(width: usize, height: usize) -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(true)),
            width,
            height,
        }
    }

    fn set_active(&mut self, is_active: bool) {
        self.is_active.store(is_active, Ordering::Relaxed);
    }
}

struct App {
    backend: Option<RealSenseBackend>,
    status: Message,

    devices_model: DevicesModel,
    devices_combo_box: DevicesComboBox,
    device_mode_panel: DeviceModePanel,

    camera_status: CameraStatus,
    cmd_tx: Sender<actions::CameraAction>,
    cmd_rx: Receiver<actions::CameraAction>,
    frame_tx: Sender<Frame>,
    frame_rx: Receiver<Frame>,

    camera_view: CameraView,
    // TODO: separate widget
    // color_image: Option<egui::ColorImage>,
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

        let (cmd_tx, cmd_rx) = crossbeam_channel::bounded::<actions::CameraAction>(1);
        let (frame_tx, frame_rx) = crossbeam_channel::bounded::<Frame>(1);

        let mut result = Self {
            backend,
            status: Message::none(),

            devices_model,
            devices_combo_box,
            device_mode_panel: DeviceModePanel::new(false),

            camera_status: CameraStatus::stopped(),
            cmd_tx,
            cmd_rx,
            frame_tx,
            frame_rx,
            camera_view: CameraView::new(),

            fatal_error,
        };

        let apply = confy::load_path("config.toml")
            .map_err(|e| e.to_string())
            .and_then(|config| result.apply_config(config).map_err(|e| e.to_string()));
        if let Err(e) = apply {
            result.status = Message::error(format!("Unable to apply conifg: {}", e));
        };

        result
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
            let size = ui.max_rect().size();
            self.camera_view.show(ui, Some(size));
        });

        actions
    }

    fn execute_actions(&mut self, actions: Vec<Action>, ctx: &egui::Context) {
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
                match self.start_camera(ctx) {
                    Ok(status) => {
                        log::info!("Camera started successfully");
                        self.device_mode_panel.set_camera_active(true);
                        self.camera_status = status;
                    }
                    Err(e) => self.status = Message::error(e.to_string()),
                };
            }
            Action::StopCamera => {
                info!("Action::StopCamera");
                self.camera_status.set_active(false);
                self.device_mode_panel.set_camera_active(false);
                // TODO: check return
                self.cmd_tx.try_send(CameraAction::Stop);
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
            Action::SelectFormat { format } => {
                info!("Action::SelectFormat {}", format);
                self.select_format(format);
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

    fn select_format(&mut self, format: PixelFormat) {
        let ok = self.devices_model.select_format(format);
        if let Err(e) = ok {
            self.status = Message::error(e.to_string());
        }
    }

    fn start_camera(&mut self, egui_ctx: &egui::Context) -> Result<CameraStatus> {
        let rs_ctx = self.backend.as_ref().map(|b| b.context());
        let Some(rs_ctx) = rs_ctx else {
            anyhow::bail!("Unable to start camera. No valid realsense2 context found");
        };

        let serial = self.devices_model.selected_device_serial();
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

        let format = self.devices_model.selected_format();
        let Some(format) = format else {
            anyhow::bail!("Unable to start camera. Select the format first");
        };

        let camera = Camera::new(serial, stream, format, mode, rs_ctx);

        match camera {
            Ok(c) => {
                start_capture_thread(
                    c,
                    self.cmd_rx.clone(),
                    self.frame_tx.clone(),
                    egui_ctx.clone(),
                );
            }
            Err(e) => anyhow::bail!(e),
        };

        Ok(CameraStatus::started(mode.width, mode.height))
    }

    fn update_frame(&mut self, ctx: &egui::Context) {
        let mut latest_frame: Option<Frame> = None;
        while let Ok(f) = self.frame_rx.try_recv() {
            latest_frame = Some(f);
        }

        if let Some(f) = latest_frame {
            self.camera_view.update_frame(
                ctx,
                &f.data,
                self.camera_status.width,
                self.camera_status.height,
                PixelFormat::Rgb8,
            );
        };
    }

    fn export_config(&self) -> Config {
        Config {
            devices_model: self.devices_model.export_config(),
        }
    }

    fn apply_config(&mut self, config: Config) -> anyhow::Result<()> {
        self.devices_model.apply_config(config.devices_model)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.update_frame(ctx);

        let actions = self.show_ui(ctx);
        self.execute_actions(actions, ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let cfg = self.export_config();
        let result = confy::store_path("config.toml", cfg);

        if let Err(e) = result {
            log::error!("error: {e}");
        }
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

fn start_capture_thread(
    mut camera: Camera,
    cmd_rx: Receiver<actions::CameraAction>,
    frame_tx: Sender<Frame>,
    egui_ctx: egui::Context,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            select! {
                recv(cmd_rx) -> msg => {
                    match msg {
                        Ok(CameraAction::Stop) | Err(_) => {
                            log::info!("Stop command received");
                            break;
                        }
                    }
                }

                default(Duration::from_millis(0)) => {}
            }

            let frame = match camera.wait_for_frames() {
                Ok(f) => f,
                Err(e) => {
                    log::error!("Camera frame error: {}", e);
                    continue;
                }
            };

            if frame_tx.try_send(frame).is_err() {
                continue;
            }

            egui_ctx.request_repaint();
        }
    })
}

fn drain_receiver<T>(rx: &Receiver<T>) -> usize {
    let mut n = 0;

    while rx.try_recv().is_ok() {
        n += 1;
    }

    n
}
