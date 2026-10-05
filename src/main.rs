#![allow(dead_code)]

mod app;
mod engine;
mod io;
mod model;
mod ui;

use app::JerichoApp;
use eframe::NativeOptions;
use egui::Vec2;
use std::sync::Arc;

fn load_app_icon() -> Option<Arc<egui::IconData>> {
    let img_bytes = include_bytes!("../assets/icon-256.png");
    if let Ok(img) = image::load_from_memory(img_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(Arc::new(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }))
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    let initial_file = std::env::args().nth(1);

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Jericho Spreadsheet")
        .with_inner_size(Vec2::new(1280.0, 800.0))
        .with_min_inner_size(Vec2::new(680.0, 480.0))
        .with_active(true);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Jericho Spreadsheet",
        native_options,
        Box::new(move |cc| Ok(Box::new(JerichoApp::new(cc, initial_file)))),
    )
}
