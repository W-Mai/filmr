use filmr::{process_image, process_image_async, FilmMetrics, FilmStock, SimulationConfig};
use image::RgbImage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Task {
    Process {
        image_data: Vec<u8>,
        width: u32,
        height: u32,
        film: FilmStock,
        config: SimulationConfig,
        is_preview: bool,
    },
}

#[derive(Serialize, Deserialize)]
pub enum WorkerResult {
    ProcessDone {
        image_data: Vec<u8>,
        width: u32,
        height: u32,
        metrics: Box<FilmMetrics>,
        is_preview: bool,
    },
    Error(String),
}

pub fn process_image_with_metrics(
    image: &RgbImage,
    film: &FilmStock,
    config: &SimulationConfig,
) -> (RgbImage, FilmMetrics) {
    let processed = process_image(image, film, config);
    let metrics = FilmMetrics::analyze(&processed);
    (processed, metrics)
}

pub async fn process_image_with_metrics_async(
    image: &RgbImage,
    film: &FilmStock,
    config: &SimulationConfig,
) -> (RgbImage, FilmMetrics) {
    let processed = process_image_async(image, film, config).await;
    let metrics = FilmMetrics::analyze(&processed);
    (processed, metrics)
}

pub fn process_task_image_data(
    image_data: Vec<u8>,
    width: u32,
    height: u32,
    film: FilmStock,
    config: SimulationConfig,
    is_preview: bool,
) -> WorkerResult {
    if let Some(img) = RgbImage::from_raw(width, height, image_data) {
        let (processed, metrics) = process_image_with_metrics(&img, &film, &config);
        let width = processed.width();
        let height = processed.height();
        WorkerResult::ProcessDone {
            image_data: processed.into_raw(),
            width,
            height,
            metrics: Box::new(metrics),
            is_preview,
        }
    } else {
        WorkerResult::Error("Failed to create image buffer".to_string())
    }
}

pub async fn process_task_image_data_async(
    image_data: Vec<u8>,
    width: u32,
    height: u32,
    film: FilmStock,
    config: SimulationConfig,
    is_preview: bool,
) -> WorkerResult {
    if let Some(img) = RgbImage::from_raw(width, height, image_data) {
        let (processed, metrics) = process_image_with_metrics_async(&img, &film, &config).await;

        let width = processed.width();
        let height = processed.height();
        WorkerResult::ProcessDone {
            image_data: processed.into_raw(),
            width,
            height,
            metrics: Box::new(metrics),
            is_preview,
        }
    } else {
        WorkerResult::Error("Failed to create image buffer".to_string())
    }
}
