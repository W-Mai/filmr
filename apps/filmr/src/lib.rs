#[cfg(feature = "cli")]
pub mod cli;

pub mod config;
pub mod exif_utils;
pub mod types;

#[cfg(target_arch = "wasm32")]
pub mod bridge;
#[cfg(target_arch = "wasm32")]
pub mod worker;

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    let window = match web_sys::window() {
        Some(window) => window,
        None => return Ok(()),
    };

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    log::info!("App starting...");

    let navigator = window.navigator();
    let concurrency = navigator.hardware_concurrency() as usize;
    log::info!("Creating Compute Worker... before");
    wasm_bindgen_futures::JsFuture::from(wasm_bindgen_rayon::init_thread_pool(concurrency)).await?;
    log::info!("Creating Compute Worker... after");

    let web_options = eframe::WebOptions::default();

    let document = window.document().expect("No document");

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
