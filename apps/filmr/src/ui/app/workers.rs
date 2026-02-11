//! Worker thread types and logic for async image processing and loading.

use filmr::{FilmMetrics, FilmStock, SimulationConfig};
use image::{DynamicImage, RgbImage};
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

/// Spawns a thread (Native) or a Rayon task (WASM) to run the given closure.
///
/// On Native, this uses `std::thread::spawn`.
/// On WASM, this uses `rayon::spawn`, assuming `wasm-bindgen-rayon` has been initialized.
pub fn spawn_thread<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        thread::spawn(f);
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(target_arch = "wasm32")]
        use std::any::Any;
        log::info!("Worker started {:?}", f.type_id());
        rayon::spawn(f);
    }
}

/// Request to process an image with film simulation.
pub struct ProcessRequest {
    pub image: Arc<RgbImage>,
    pub film: FilmStock,
    pub config: SimulationConfig,
    pub is_preview: bool,
}

/// Result of image processing.
pub struct ProcessResult {
    pub image: RgbImage,
    pub metrics: FilmMetrics,
    pub is_preview: bool,
}

/// Request to load an image from file or bytes.
pub struct LoadRequest {
    pub path: Option<PathBuf>,
    pub bytes: Option<Arc<[u8]>>,
    pub stock: Option<FilmStock>,
}

/// Data returned from successful image load.
pub struct LoadResultData {
    pub image: DynamicImage,
    pub texture_data: egui::ColorImage,
    pub metrics: FilmMetrics,
    pub preview: Arc<RgbImage>,
    pub preview_texture_data: egui::ColorImage,
    pub estimated_exposure: Option<f32>,
}

/// Result of image loading operation.
pub struct LoadResult {
    pub path: Option<PathBuf>,
    pub result: Result<LoadResultData, String>,
}

#[cfg(not(target_arch = "wasm32"))]
use crate::types::process_image_with_metrics;

/// Process worker logic for native builds.
#[cfg(not(target_arch = "wasm32"))]
pub fn process_worker_logic(req: ProcessRequest) -> ProcessResult {
    let (processed, metrics) = process_image_with_metrics(&req.image, &req.film, &req.config);
    ProcessResult {
        image: processed,
        metrics,
        is_preview: req.is_preview,
    }
}

/// Load worker logic - handles image loading with EXIF orientation.
pub fn load_worker_logic(req: LoadRequest) -> LoadResult {
    use crate::exif_utils::{apply_exif_orientation, read_exif_orientation};
    use egui::ColorImage;
    use filmr::estimate_exposure_time;
    use image::imageops::FilterType;
    use std::io::{BufReader, Cursor};

    // Read EXIF orientation before loading image
    let orientation = if let Some(bytes) = &req.bytes {
        let mut cursor = Cursor::new(bytes.as_ref());
        read_exif_orientation(&mut cursor)
    } else if let Some(path) = &req.path {
        std::fs::File::open(path)
            .ok()
            .map(|f| {
                let mut reader = BufReader::new(f);
                read_exif_orientation(&mut reader)
            })
            .unwrap_or(1)
    } else {
        1
    };

    let img_result = if let Some(bytes) = &req.bytes {
        image::load_from_memory(bytes)
    } else if let Some(path) = &req.path {
        image::open(path)
    } else {
        Err(image::ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No image source provided",
        )))
    };

    let result = match img_result {
        Ok(img) => {
            // Apply EXIF orientation transform
            let img = apply_exif_orientation(img, orientation);

            let rgb = img.to_rgb8();
            let metrics = FilmMetrics::analyze(&rgb);
            let texture_data = ColorImage::from_rgb(
                [rgb.width() as _, rgb.height() as _],
                rgb.as_flat_samples().as_slice(),
            );

            let width = img.width();
            let height = img.height();
            let preview_rgb = if width > 2048 || height > 2048 {
                img.resize(2048, 2048, FilterType::Lanczos3).to_rgb8()
            } else {
                rgb.clone()
            };
            let preview_texture_data = ColorImage::from_rgb(
                [preview_rgb.width() as _, preview_rgb.height() as _],
                preview_rgb.as_flat_samples().as_slice(),
            );

            let estimated_exposure = req
                .stock
                .map(|stock| estimate_exposure_time(&preview_rgb, &stock));

            Ok(LoadResultData {
                image: img,
                texture_data,
                metrics,
                preview: Arc::new(preview_rgb),
                preview_texture_data,
                estimated_exposure,
            })
        }
        Err(e) => Err(e.to_string()),
    };

    LoadResult {
        path: req.path,
        result,
    }
}
