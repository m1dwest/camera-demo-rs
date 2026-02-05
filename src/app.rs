use anyhow::{Result, ensure};
use eframe::egui;

use log::{debug, error, info};
use realsense_rust as rs;

use crate::core::Camera;
use crate::core::RealSenseBackend;
use crate::ui::devices_combo_box::DevicesComboBox;
use crate::ui::status_bar::Message;

struct App {
    backend: RealSenseBackend,
    status: Message,
    devices_combo_box: DevicesComboBox,
    // fatal_error: Option<String>,
}

impl App {
    fn new() -> Result<Self> {
        let backend = RealSenseBackend::new()?;

        // TODO: NOne
        let devices_combo_box = DevicesComboBox::new("Available devices", None);

        let selected_sn = devices_combo_box.selected_sn();
        let selected_mode = devices_combo_box.selected_mode();

        Ok(Self {
            backend,
            status: Message::none(),
            devices_combo_box,
        })
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
                    // TODO: debug
                    // self.camera = if sn.is_empty() {
                    //     None
                    // } else {
                    //     Camera::new(&sn).ok()
                    // };
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
