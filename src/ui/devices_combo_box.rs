use eframe::egui;

pub enum Action {
    None,
    Changed { sn: String },
}

pub struct DevicesComboBox {
    label: String,
    selected: Option<String>,
    items: Vec<(String, String)>,
}

impl DevicesComboBox {
    pub fn new(label: &str, items: Vec<(String, String)>) -> Self {
        Self {
            label: label.to_owned(),
            selected: if !items.is_empty() {
                items.first().map(|(sn, _)| sn.clone())
            } else {
                None
            },
            items,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Action {
        let mut action = Action::None;

        let selected: &str = self
            .selected
            .as_deref()
            .and_then(|selected_sn| {
                self.items
                    .iter()
                    .find(|(sn, _)| sn == selected_sn)
                    .map(|(_, name)| name.as_str())
            })
            .unwrap_or("");

        egui::ComboBox::from_label(self.label.clone())
            .selected_text(selected)
            .show_ui(ui, |ui| {
                self.items.iter().for_each(|(sn, name)| {
                    let is_selected = self.selected.as_deref() == Some(sn.as_str());
                    if ui.selectable_label(is_selected, name).clicked() {
                        self.selected = Some(sn.clone());
                        action = Action::Changed { sn: sn.clone() };
                    }
                });
            });

        action
    }

    pub fn selected_sn(&self) -> Option<String> {
        self.selected.clone()
    }
}
