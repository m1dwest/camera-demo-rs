use eframe::egui;

use crate::app::actions::Action;
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
                for sensor in model.sensors() {
                    let is_selected = model
                        .selected_sensor()
                        .as_ref()
                        .is_some_and(|s| s == sensor);

                    if ui.selectable_label(is_selected, sensor).clicked() {
                        actions.push(Action::SelectSensor {
                            sensor: sensor.clone(),
                        });
                    }
                }
            });

        let stream_text = model
            .selected_stream()
            .map(|s| s.to_string())
            .unwrap_or("Select stream".to_owned());

        egui::ComboBox::from_label("Stream")
            .selected_text(stream_text)
            .show_ui(ui, |ui| {
                for &stream in model.streams() {
                    let is_selected = model.selected_stream().is_some_and(|s| s == stream);

                    if ui
                        .selectable_label(is_selected, stream.to_string())
                        .clicked()
                    {
                        actions.push(Action::SelectStream { stream });
                    }
                }
            });

        let res_text = model
            .selected_mode()
            .map(|mode| mode.to_string())
            .unwrap_or("Select resolution".to_owned());

        egui::ComboBox::from_label("Resolution")
            .selected_text(res_text)
            .show_ui(ui, |ui| {
                let selected_mode = model.selected_mode();

                for mode in model.modes() {
                    let is_selected = selected_mode.as_ref().is_some_and(|m| m == &mode);

                    if ui.selectable_label(is_selected, mode.to_string()).clicked() {
                        actions.push(Action::SelectMode { mode: mode.clone() });
                    }
                }
            });

        actions
    }
}
