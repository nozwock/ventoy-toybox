mod gui;
use crate::gui::app;
use eframe::egui;

fn main() {
    // let native_options = eframe::NativeOptions::default();
    let options = eframe::NativeOptions {
        // Hide the OS-specific "chrome" around the window:
        // decorated: true,
        // To have rounded corners we need transparency:
        // transparent: true,
        min_window_size: Some(egui::vec2(350.0, 100.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Ventoy Toybox App",
        options,
        Box::new(|cc| Box::new(app::ToyboxApp::new(cc))),
    );
}
