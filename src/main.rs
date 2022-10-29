mod core;
mod gui;
use crate::gui::app;
use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(700.0, 500.0)),
        min_window_size: Some(egui::vec2(500.0, 250.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Ventoy Toybox",
        options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    );
}
