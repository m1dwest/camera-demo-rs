use anyhow::{Result, ensure};
use eframe::egui;

use log::{debug, error, info};
use realsense_rust as rs;

use crate::core::Camera;
use crate::ui::devices_combo_box::DevicesComboBox;
use crate::ui::status_bar::Message;

struct App {
    camera: Option<Camera>,
    status: Message,
    devices_combo_box: DevicesComboBox,
}

impl App {
    fn new() -> Self {
        let (camera, status) = match Camera::new() {
            Ok(c) => (Some(c), Message::none()),
            Err(e) => (None, Message::error(format!("{:#}", e))),
        };

        let devices_combo_box = DevicesComboBox::new(
            "Available devices",
            if let Some(camera) = &camera {
                camera.devices.get_names()
            } else {
                Vec::new()
            },
        );

        if let Some(sn) = devices_combo_box.selected_sn() {
            App::init_camera(sn);
        }

        Self {
            camera,
            status,
            devices_combo_box,
        }
    }

    fn init_camera(sn: String) {
        //
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.status = Message::warn("Warning message");

        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("Control panel");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::new().show(ui, |ui| {
                let action = self.devices_combo_box.show(ui);
                if let crate::ui::devices_combo_box::Action::Changed { sn } = action {
                    App::init_camera(sn);
                }
            });
        });
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
