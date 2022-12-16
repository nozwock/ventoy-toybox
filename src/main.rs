#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// https://github.com/emilk/egui/pull/1008
mod core;
mod defines;
mod gui;
use crate::gui::app;
use eframe::epaint::vec2;

fn main() {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(700.0, 500.0)),
        min_window_size: Some(vec2(500.0, 250.0)),
        #[cfg(feature = "app-icon")]
        icon_data: Some(raw_to_icon_data(defines::APP_ICON).unwrap()),
        ..Default::default()
    };
    eframe::run_native(
        crate::defines::APP_NAME,
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    );
}

#[cfg(feature = "app-icon")]
fn raw_to_icon_data(raw: &[u8]) -> image::ImageResult<eframe::IconData> {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = image::load_from_memory(raw)?.to_rgba8();
        let (width, height) = icon.dimensions();
        (icon.into_raw(), width, height)
    };

    Ok(eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    })
}
