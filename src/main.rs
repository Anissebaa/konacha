#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod app;
mod config;
mod error;
mod platform;
mod scripting;
mod storage;
mod theme;
mod ui;

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    ).init();

    let prefs = config::prefs::Preferences::load();
    let size = prefs.window_size;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_title("Konacha")
        .with_inner_size(size)
        .with_min_inner_size([880.0, 560.0])
        .with_decorations(true),
        ..Default::default()
    };

    eframe::run_native(
        "Konacha",
        native_options,
        Box::new(|cc| Ok(Box::new(app::KonachaApp::new(cc)))),
    )
}
