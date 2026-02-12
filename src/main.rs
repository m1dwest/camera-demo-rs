#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod app;
mod core;
mod ui;

fn setup() {
    let my_crate = env!("CARGO_PKG_NAME").replace('-', "_");
    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(log::LevelFilter::Error)
        .filter_module(&my_crate, log::LevelFilter::Trace)
        .init();
}

fn main() -> eframe::Result {
    setup();
    app::run()
}
