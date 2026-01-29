#[cfg(feature = "cli")]
pub mod cli;

pub mod config;
pub mod cus_component;

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    let document = web_sys::window()
        .expect("No window")
        .document()
        .expect("No document");

    let canvas = document
        .get_element_by_id("the_canvas_id")
        .expect("Failed to find the_canvas_id")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("the_canvas_id was not a HtmlCanvasElement");

    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|cc| Ok(Box::new(ui::app::FilmrApp::new(cc)))),
        )
        .await
}
