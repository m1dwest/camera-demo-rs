use eframe::egui;

use crate::actions::Action;

pub struct DevicesComboBox {
    label: String,
    devices: Vec<crate::core::Device>,
    selected_serial: String,
}

const NONE_LABEL: &str = "None";
const HEIGHT: f32 = 40.0;

fn get_selected_serial(devices: &[crate::core::Device], current: Option<&str>) -> String {
    if let Some(serial) = current
        && serial == NONE_LABEL
    {
        return NONE_LABEL.to_owned();
    }

    if let Some(serial) = devices
        .iter()
        .find(|d| d.serial.as_deref() == current)
        .and_then(|d| d.serial.clone())
    {
        return serial;
    }

    devices
        .first()
        .and_then(|d| d.serial.clone())
        .unwrap_or(NONE_LABEL.to_owned())
}

impl DevicesComboBox {
    pub fn new(label: &str, devices: Vec<crate::core::Device>) -> Self {
        let selected_serial = get_selected_serial(&devices, None);

        Self {
            label: label.to_owned(),
            devices,
            selected_serial,
        }
    }

    pub fn refresh_device_list(&mut self, devices: Vec<crate::core::Device>) {
        self.selected_serial = get_selected_serial(&devices, Some(self.selected_serial.as_str()));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<Action> {
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

                            let combo_actions = self.show_combo_box(ui);
                            actions.extend(combo_actions);
                        });
                    });
            },
        );

        actions
    }

    fn show_combo_box(&mut self, ui: &mut egui::Ui) -> Vec<Action> {
        let mut actions = Vec::new();

        let selected = self
            .name_from_serial(&self.selected_serial)
            .unwrap_or(NONE_LABEL);

        egui::ComboBox::from_label(self.label.clone())
            .selected_text(selected)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(self.selected_serial == NONE_LABEL, NONE_LABEL)
                    .clicked()
                {
                    self.selected_serial = NONE_LABEL.to_owned();
                    actions.push(Action::DisableCamera);
                }

                self.devices.iter().for_each(|device| {
                    let (is_selected, name) = if let Some(serial) = &device.serial {
                        (
                            self.selected_serial.as_str() == serial.as_str(),
                            device.name.as_deref().unwrap_or("Unknown name"),
                        )
                    } else {
                        (false, "Invalid device")
                    };

                    if ui.selectable_label(is_selected, name).clicked()
                        && let Some(serial) = &device.serial
                    {
                        self.selected_serial = serial.clone();
                        actions.push(Action::ChangeCamera {
                            serial: serial.clone(),
                        });
                    }
                });
            });

        actions
    }

    fn name_from_serial(&self, serial: &str) -> Option<&str> {
        self.devices
            .iter()
            .find(|d| d.serial.as_deref() == Some(serial))
            .and_then(|d| d.name.as_deref())
    }
}
