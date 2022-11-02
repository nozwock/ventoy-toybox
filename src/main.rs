#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// https://github.com/emilk/egui/pull/1008
mod core;
mod defines;
mod gui;
use crate::gui::app;
use eframe::epaint::vec2;

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(700.0, 500.0)),
        min_window_size: Some(vec2(500.0, 250.0)),
        ..Default::default()
    };
    eframe::run_native(
        crate::defines::APP_NAME,
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    );
}
