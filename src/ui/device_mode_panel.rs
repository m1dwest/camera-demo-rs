use eframe::egui;

use crate::actions::Action;
use crate::core::DevicesModel;

pub struct DeviceModePanel {}

impl DeviceModePanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui, model: &DevicesModel) -> Vec<Action> {
        let mut actions = Vec::new();

        let sensor_text = model
            .selected_sensor()
            .unwrap_or("Select sensor")
            .to_owned();

        egui::ComboBox::from_label("Sensor")
            .selected_text(&sensor_text)
            .show_ui(ui, |ui| {
                for c in model.sensors() {
                    let is_selected = model
                        .selected_sensor()
                        .as_ref()
                        .is_some_and(|sensor| sensor == c);

                    if ui.selectable_label(is_selected, c).clicked() {
                        actions.push(Action::SelectSensor { sensor: c.clone() });
                    }
                }
            });

        actions
    }
}
