use eframe::egui;

use crate::app::actions::Action;
use crate::core::Device;
use crate::core::devices_model::DevicesModel;

pub struct DevicesComboBox {
    label: String,
}

pub struct Item {
    name: String,
    serial: String,
}

const HEIGHT: f32 = 40.0;

fn decorated_name(device: &Device) -> String {
    device
        .serial
        .is_some()
        .then_some(device.name.clone())
        .unwrap_or(format!("Invalid: {}", device.name))
}

impl DevicesComboBox {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, model: &DevicesModel) -> Vec<Action> {
        let mut actions = Vec::new();

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(4, 8))
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            if ui.button("↻").clicked() {
                                actions.push(Action::RefreshDevices);
                            }

                            let combo_actions = self.show_combo_box(ui, model);
                            actions.extend(combo_actions);
                        });
                    });
            },
        );

        actions
    }

    fn show_combo_box(&mut self, ui: &mut egui::Ui, model: &DevicesModel) -> Vec<Action> {
        let mut actions = Vec::new();

        let selected_text = model
            .selected_device()
            .map(decorated_name)
            .unwrap_or("Select device".to_owned());

        egui::ComboBox::from_label(self.label.clone())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                model.devices.iter().for_each(|device| {
                    let name = decorated_name(device);

                    let is_device_selected = match (
                        model.selected_device().and_then(|d| d.serial.as_ref()),
                        &device.serial,
                    ) {
                        (Some(a), Some(b)) => a == b,
                        _ => false,
                    };

                    if !ui.selectable_label(is_device_selected, name).clicked() {
                        return;
                    }

                    if is_device_selected {
                        return;
                    }

                    let Some(serial) = device.serial.as_deref() else {
                        return;
                    };

                    actions.push(Action::ChangeCamera {
                        serial: serial.to_owned(),
                    });
                });
            });

        actions
    }
}
