#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod camera;
mod types;

use anyhow::{Result, ensure};
use eframe::egui;

use log::{debug, error, info};
use realsense_rust as rs;
use types::Message;

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui app",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}

struct App {
    camera: Option<camera::Camera>,
    status: Message,

    camera_selected: usize,
}

impl App {
    fn new() -> Self {
        let (camera, status) = match camera::Camera::new() {
            Ok(c) => (Some(c), Message::None),
            Err(e) => (None, Message::Error(format!("{:#}", e))),
        };

        Self {
            camera,
            status,
            camera_selected: 0,
        }
    }

    fn status_color(&self) -> egui::Color32 {
        match self.status {
            Message::None | Message::Info(_) => egui::Color32::from_rgb(40, 90, 200),
            Message::Warn(_) => egui::Color32::from_rgb(230, 190, 40),
            Message::Error(_) => egui::Color32::from_rgb(200, 50, 50),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(&ctx, |ui| {
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

        egui::TopBottomPanel::bottom("status_panel")
            .frame(
                egui::Frame::none()
                    .fill(self.status_color())
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(&ctx, |ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);

                ui.horizontal(|ui| {
                    let fps = ctx.input(|i| i.stable_dt).recip();
                    ui.label(format!("FPS: {:.1}", fps));
                });
            });

        egui::SidePanel::left("control_panel").show(&ctx, |ui| {
            ui.heading("Control panel");
        });
    }
}
