use serde::Serialize;

#[derive(Serialize)]
pub struct Config {
    pub devices_model: crate::core::devices_model::Config,
}
