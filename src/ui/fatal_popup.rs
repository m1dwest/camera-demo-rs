use eframe::egui;

pub fn Show(ctx: &egui::Context, error: &str) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.4);
            ui.heading(error)
        });
    });
}
