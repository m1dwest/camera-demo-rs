use anyhow::{Result, ensure};
use eframe::egui;

use log::{debug, error, info};
use realsense_rust as rs;

use crate::core::Camera;
use crate::ui::status_bar::Message;

struct App {
    camera: Option<Camera>,
    status: Message,

    camera_selected: usize,
}

impl App {
    fn new() -> Self {
        let (camera, status) = match Camera::new() {
            Ok(c) => (Some(c), Message::none()),
            Err(e) => (None, Message::error(format!("{:#}", e))),
        };

        Self {
            camera,
            status,
            // BUG: unstable id
            camera_selected: 0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::new().show(ui, |ui| {
                let mut selected: usize = 0;
                // egui::ComboBox::from_label("Available devices").show_ui(ui, |ui| {
                //     let names = self.camera.devices.names();
                //     for (i, name) in names.iter().enumerate() {
                //         ui.selectable_value(&mut selected, i, *name);
                //     }
                // });
            });
            ui.heading("Camera view");
        });

        self.status = Message::warn("Warning message");
        crate::ui::status_bar::Show(ctx, &self.status);

        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("Control panel");
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
