use eframe::egui;

pub enum Action {
    None,
    Change { serial: String },
    Disable,
}

pub struct DevicesComboBox {
    label: String,
    devices: Vec<crate::core::Device>,
    selected_serial: String,
}

impl DevicesComboBox {
    pub fn new(label: &str, devices: Vec<crate::core::Device>) -> Self {
        let selected_serial = if let Some(serial) = devices.first().and_then(|d| d.serial.clone()) {
            serial
        } else {
            "None".to_owned()
        };

        Self {
            label: label.to_owned(),
            devices,
            selected_serial,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Action {
        let mut action = Action::None;

        let selected = self
            .name_from_serial(&self.selected_serial)
            .unwrap_or("None");

        egui::ComboBox::from_label(self.label.clone())
            .selected_text(selected)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(self.selected_serial == "None", "None")
                    .clicked()
                {
                    self.selected_serial = "None".to_owned();
                    action = Action::Disable;
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
                        action = Action::Change {
                            serial: serial.clone(),
                        };
                    }
                });
            });

        action
    }

    fn name_from_serial(&self, serial: &str) -> Option<&str> {
        self.devices
            .iter()
            .find(|d| d.serial.as_deref() == Some(serial))
            .and_then(|d| d.name.as_deref())
    }

    // pub fn selected_serial(&self) -> Option<String> {
    //     self.selected_serial.clone()
    // }
    //
    // pub fn selected_mode(&self) -> Option<crate::core::devices::Mode> {
    //     None
    // }
}
