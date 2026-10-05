#![allow(dead_code)]

mod app;
mod engine;
mod io;
mod model;
mod ui;

use app::JerichoApp;
use eframe::NativeOptions;
use egui::Vec2;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Jericho Spreadsheet")
            .with_inner_size(Vec2::new(1280.0, 800.0))
            .with_min_inner_size(Vec2::new(680.0, 480.0))
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Jericho Spreadsheet",
        native_options,
        Box::new(|cc| Ok(Box::new(JerichoApp::new(cc)))),
    )
}
