pub mod app;
pub mod panels;

#[cfg(not(target_arch = "wasm32"))]
use app::FilmrApp;
#[cfg(not(target_arch = "wasm32"))]
use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Filmr GUI",
        options,
        Box::new(|cc| Ok(Box::new(FilmrApp::new(cc)))),
    )
}
