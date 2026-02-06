use eframe::egui;

pub enum MessageType {
    None,
    Info,
    Warn,
    Error,
}

pub struct Message {
    pub message_type: MessageType,
    pub text: String,

    color: egui::Color32,
    icon: char,
}

impl Message {
    pub fn none() -> Self {
        Self {
            message_type: MessageType::None,
            text: String::new(),
            color: egui::Color32::from_rgb(40, 90, 200),
            icon: ' ',
        }
    }

    pub fn _info(message: impl Into<String>) -> Self {
        Self {
            message_type: MessageType::Info,
            text: message.into(),
            color: egui::Color32::from_rgb(40, 90, 200),
            icon: '🛈',
        }
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self {
            message_type: MessageType::Warn,
            text: message.into(),
            color: egui::Color32::from_rgb(230, 190, 40),
            icon: '⚠',
        }
    }

    pub fn _error(message: impl Into<String>) -> Self {
        Self {
            message_type: MessageType::Error,
            text: message.into(),
            color: egui::Color32::from_rgb(200, 50, 50),
            icon: '⚠',
        }
    }
}

pub fn show(ctx: &egui::Context, message: &Message) -> egui::Response {
    egui::TopBottomPanel::bottom("status_panel")
        .frame(
            egui::Frame::new()
                .fill(message.color)
                .inner_margin(egui::Margin::symmetric(8, 4)),
        )
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);

            ui.horizontal(|ui| {
                ui.label(format!("{} {}", &message.icon, &message.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fps = ctx.input(|i| i.stable_dt).recip();
                    ui.label(format!("FPS: {:.1}", fps));
                });
            });
        })
        .response
}
