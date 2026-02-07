use eframe::egui;

use crate::actions::Action;
use crate::core::devices_model::{DevicesModel, DevicesModelItem};

pub struct DevicesComboBox {
    label: String,
}

pub struct Item {
    name: String,
    serial: String,
}

const HEIGHT: f32 = 40.0;

fn decorated_name(item: &DevicesModelItem) -> String {
    if item.serial.is_some() {
        item.name.to_owned()
    } else {
        format!("Invalid: {}", item.name)
    }
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
                                actions.push(Action::RefreshDeviceList);
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

        let selected_text = decorated_name(model.current_item());

        egui::ComboBox::from_label(self.label.clone())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                model.items.iter().enumerate().for_each(|(i, item)| {
                    let is_selected = i == model.current_index();
                    let name = decorated_name(item);

                    if !ui.selectable_label(is_selected, name).clicked() {
                        return;
                    }

                    let Some(serial) = item.serial.as_deref() else {
                        return;
                    };

                    let is_already_selected =
                        model.current_item().serial.as_deref() == Some(serial);
                    if is_already_selected {
                        return;
                    }

                    actions.push(Action::ChangeCamera {
                        serial: serial.to_owned(),
                    });
                });
            });

        actions
    }
}
