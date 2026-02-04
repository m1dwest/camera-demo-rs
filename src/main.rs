#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod ui;

fn setup() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

fn main() -> eframe::Result {
    setup();
    app::run()
}
