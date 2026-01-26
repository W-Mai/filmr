pub mod app;
pub mod panels;

use app::FilmrApp;
use eframe::egui;

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Filmr GUI Example",
        options,
        Box::new(|cc| Ok(Box::new(FilmrApp::new(cc)))),
    )
}
