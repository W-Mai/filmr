//! FilmrApp - Main application state and initialization.

mod io;
mod processing;
mod update;
pub mod workers;

pub use crate::config::{AppMode, ConfigManager, UxMode};

use egui::{TextureHandle, Vec2};
use filmr::film::FilmStockCollection;
use filmr::{
    light_leak::LightLeakConfig, presets, FilmMetrics, FilmStock, OutputMode, SimulationConfig,
    WhiteBalanceMode,
};
use flume::{unbounded, Receiver, Sender};
use image::{DynamicImage, RgbImage};
use std::path::PathBuf;
use std::sync::Arc;

use workers::{
    load_worker_logic, spawn_thread, LoadRequest, LoadResult, ProcessRequest, ProcessResult,
};

#[cfg(target_arch = "wasm32")]
use crate::bridge::ComputeBridge;
#[cfg(target_arch = "wasm32")]
use crate::types::{Task, WorkerResult};

/// Main application state for Filmr.
pub struct FilmrApp {
    // State
    pub original_image: Option<DynamicImage>,
    pub preview_image: Option<Arc<RgbImage>>,
    pub developed_image: Option<DynamicImage>,
    pub processed_texture: Option<TextureHandle>,
    pub original_texture: Option<TextureHandle>,
    pub metrics_original: Option<FilmMetrics>,
    pub metrics_preview: Option<FilmMetrics>,
    pub metrics_developed: Option<FilmMetrics>,
    pub source_path: Option<PathBuf>,
    pub source_exif: Option<little_exif::metadata::Metadata>,

    // Async Processing
    pub(crate) tx_req: Sender<ProcessRequest>,
    pub(crate) rx_res: Receiver<ProcessResult>,
    pub is_processing: bool,

    // Async Loading
    pub(crate) tx_load: Sender<LoadRequest>,
    pub(crate) rx_load: Receiver<LoadResult>,
    pub is_loading: bool,

    // View State
    pub zoom: f32,
    pub offset: Vec2,
    pub show_original: bool,
    pub show_metrics: bool,
    pub split_view: bool,
    pub split_pos: f32,

    // Parameters
    pub exposure_time: f32,
    pub gamma_boost: f32,
    pub warmth: f32,
    pub saturation: f32,

    // Halation Parameters
    pub halation_strength: f32,
    pub halation_threshold: f32,
    pub halation_sigma: f32,

    // Grain Parameters
    pub grain_alpha: f32,
    pub grain_sigma: f32,
    pub grain_roughness: f32,
    pub grain_blur_radius: f32,

    // Light Leak Parameters
    pub light_leak_config: LightLeakConfig,

    // Selection
    pub stocks: Vec<std::rc::Rc<FilmStock>>,
    pub selected_stock_idx: usize,

    pub output_mode: OutputMode,
    pub white_balance_mode: WhiteBalanceMode,
    pub white_balance_strength: f32,

    // Status
    pub status_msg: String,

    // Metrics Display Options
    pub hist_log_scale: bool,
    pub hist_clamp_zeros: bool,
    pub hist_smooth: bool,

    // App Mode
    pub mode: AppMode,
    pub ux_mode: UxMode,
    pub studio_stock: FilmStock,
    pub builtin_stock_count: usize,

    // Studio State
    pub studio_stock_idx: Option<usize>,
    pub has_unsaved_changes: bool,
    pub show_exit_dialog: bool,
    pub show_settings: bool,

    pub config_manager: Option<ConfigManager>,

    pub preset_thumbnails: std::collections::HashMap<String, TextureHandle>,
    pub(crate) tx_thumb: Sender<(String, RgbImage, SimulationConfig, FilmStock)>,
    pub(crate) rx_thumb: Receiver<(String, RgbImage)>,

    // Preset Loading (WASM)
    #[cfg(target_arch = "wasm32")]
    pub tx_preset: Sender<Vec<u8>>,
    #[cfg(target_arch = "wasm32")]
    pub rx_preset: Receiver<Vec<u8>>,
}

impl FilmrApp {
    /// Create a new FilmrApp instance.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(cc);

        let mut stocks = presets::get_all_stocks();
        let builtin_stock_count = stocks.len();
        let config_manager = ConfigManager::init();

        // Load custom stocks
        Self::load_custom_stocks(&config_manager, &mut stocks);

        // Setup channels
        let (tx_req, rx_req) = unbounded::<ProcessRequest>();
        let (tx_res, rx_res) = unbounded::<ProcessResult>();
        let (tx_load, rx_load) = unbounded::<LoadRequest>();
        let (tx_load_res, rx_load_res) = unbounded::<LoadResult>();
        let (tx_thumb, rx_thumb_internal) =
            unbounded::<(String, RgbImage, SimulationConfig, FilmStock)>();
        let (tx_thumb_res, rx_thumb_res) = unbounded::<(String, RgbImage)>();

        // Clone context for the threads
        let ctx_process = cc.egui_ctx.clone();
        let ctx_load = cc.egui_ctx.clone();
        let ctx_thumb = cc.egui_ctx.clone();

        // Spawn worker threads
        Self::spawn_process_worker(rx_req, tx_res, ctx_process);
        Self::spawn_load_worker(rx_load, tx_load_res, ctx_load);
        Self::spawn_thumbnail_worker(rx_thumb_internal, tx_thumb_res, ctx_thumb);

        #[cfg(target_arch = "wasm32")]
        let (tx_preset, rx_preset) = unbounded();

        let ux_mode = config_manager
            .as_ref()
            .map(|cm| cm.config.ux_mode)
            .unwrap_or(UxMode::Simple);

        Self {
            original_image: None,
            preview_image: None,
            developed_image: None,
            processed_texture: None,
            original_texture: None,
            metrics_original: None,
            metrics_preview: None,
            metrics_developed: None,
            source_path: None,
            source_exif: None,

            tx_req,
            rx_res,
            is_processing: false,

            tx_load,
            rx_load: rx_load_res,
            is_loading: false,

            preset_thumbnails: std::collections::HashMap::new(),
            tx_thumb,
            rx_thumb: rx_thumb_res,

            zoom: 1.0,
            offset: Vec2::ZERO,
            show_original: false,
            show_metrics: false,
            split_view: false,
            split_pos: 0.5,
            exposure_time: 1.0,
            gamma_boost: 1.0,
            warmth: 0.0,
            saturation: 1.0,

            // Default Halation params
            halation_strength: 0.0,
            halation_threshold: 0.8,
            halation_sigma: 0.02,

            // Default Grain params (will be overwritten by preset)
            grain_alpha: 0.01,
            grain_sigma: 0.01,
            grain_roughness: 0.5,
            grain_blur_radius: 0.5,

            light_leak_config: LightLeakConfig::default(),

            stocks,
            selected_stock_idx: 0, // Default to first
            output_mode: OutputMode::Positive,
            white_balance_mode: WhiteBalanceMode::Auto,
            white_balance_strength: 1.0,
            status_msg: "Drag and drop an image here to start.".to_owned(),

            hist_log_scale: false,
            hist_clamp_zeros: true,
            hist_smooth: false,

            mode: AppMode::Develop,
            ux_mode,
            studio_stock: presets::other::STANDARD_DAYLIGHT(),
            builtin_stock_count,

            studio_stock_idx: None,
            has_unsaved_changes: false,
            show_exit_dialog: false,
            show_settings: false,

            config_manager,

            #[cfg(target_arch = "wasm32")]
            tx_preset,
            #[cfg(target_arch = "wasm32")]
            rx_preset,
        }
    }

    /// Get the currently selected film stock.
    pub fn get_current_stock(&self) -> std::rc::Rc<FilmStock> {
        let index = if self.selected_stock_idx < self.stocks.len() {
            self.selected_stock_idx
        } else {
            0
        };

        self.stocks[index].clone()
    }

    // --- Private helper methods ---

    fn setup_fonts(cc: &eframe::CreationContext<'_>) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "ark-pixel".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../../static/ark-pixel-12px-monospaced-zh_cn.otf"
            ))),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "ark-pixel".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "ark-pixel".to_owned());
        cc.egui_ctx.set_fonts(fonts);
    }

    fn load_custom_stocks(
        config_manager: &Option<ConfigManager>,
        stocks: &mut Vec<std::rc::Rc<FilmStock>>,
    ) {
        if let Some(cm) = config_manager {
            if let Ok(entries) = std::fs::read_dir(&cm.config.custom_stocks_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "json") {
                        // Try collection first
                        if let Ok(file) = std::fs::File::open(&path) {
                            let reader = std::io::BufReader::new(file);
                            if let Ok(collection) =
                                serde_json::from_reader::<_, FilmStockCollection>(reader)
                            {
                                for (name, mut stock) in collection.stocks {
                                    if stock.name.is_empty() {
                                        stock.name = name;
                                    }
                                    stocks.push(std::rc::Rc::from(stock));
                                }
                            } else if let Ok(mut stock) = FilmStock::load_from_file(&path) {
                                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                                if stock.name.is_empty() {
                                    stock.name = name;
                                }
                                stocks.push(std::rc::Rc::from(stock));
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_process_worker(
        rx_req: Receiver<ProcessRequest>,
        tx_res: Sender<ProcessResult>,
        ctx: egui::Context,
    ) {
        use workers::process_worker_logic;

        spawn_thread(move || {
            while let Ok(mut req) = rx_req.recv() {
                while let Ok(newer) = rx_req.try_recv() {
                    req = newer;
                }

                let width = req.image.width();
                let height = req.image.height();
                log::info!("Native worker starting process: {}x{}", width, height);

                let res = process_worker_logic(req);

                log::info!("Native worker process done");
                let _ = tx_res.send(res);
                ctx.request_repaint();
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn_process_worker(
        rx_req: Receiver<ProcessRequest>,
        tx_res: Sender<ProcessResult>,
        ctx: egui::Context,
    ) {
        use image::RgbImage;

        let bridge = ComputeBridge::new();
        let bridge_clone = bridge.clone();

        // Request Handler
        wasm_bindgen_futures::spawn_local(async move {
            while let Ok(mut req) = rx_req.recv_async().await {
                while let Ok(newer) = rx_req.try_recv() {
                    req = newer;
                }

                let width = req.image.width();
                let height = req.image.height();
                log::info!(
                    "Sending process task to worker: {}x{}, preview={}",
                    width,
                    height,
                    req.is_preview
                );

                let task = Task::Process {
                    image_data: req.image.as_raw().clone(),
                    width,
                    height,
                    film: req.film,
                    config: req.config,
                    is_preview: req.is_preview,
                };
                bridge_clone.submit_task(task);
            }
        });

        // Result Handler
        let ctx = ctx.clone();
        let rx = bridge.result_receiver();

        wasm_bindgen_futures::spawn_local(async move {
            while let Ok(res) = rx.recv_async().await {
                match res {
                    WorkerResult::ProcessDone {
                        image_data,
                        width,
                        height,
                        metrics,
                        is_preview,
                    } => {
                        if let Some(img) = RgbImage::from_raw(width, height, image_data) {
                            let res = ProcessResult {
                                image: img,
                                metrics: *metrics,
                                is_preview,
                            };
                            let _ = tx_res.send(res);
                            ctx.request_repaint();
                        }
                    }
                    WorkerResult::Error(e) => {
                        log::error!("Worker error: {}", e);
                    }
                }
            }
        });
    }

    fn spawn_load_worker(
        rx_load: Receiver<LoadRequest>,
        tx_load_res: Sender<LoadResult>,
        ctx: egui::Context,
    ) {
        spawn_thread(move || {
            while let Ok(req) = rx_load.recv() {
                let res = load_worker_logic(req);
                let _ = tx_load_res.send(res);
                ctx.request_repaint();
            }
        });
    }

    fn spawn_thumbnail_worker(
        rx_thumb: Receiver<(String, RgbImage, SimulationConfig, FilmStock)>,
        tx_thumb_res: Sender<(String, RgbImage)>,
        ctx: egui::Context,
    ) {
        use filmr::{estimate_exposure_time, process_image};

        spawn_thread(move || {
            while let Ok(first) = rx_thumb.recv() {
                // Drain channel to get the latest batch, discard stale requests
                let mut latest: std::collections::HashMap<
                    String,
                    (RgbImage, SimulationConfig, FilmStock),
                > = std::collections::HashMap::new();
                latest.insert(first.0, (first.1, first.2, first.3));
                while let Ok((name, img, config, stock)) = rx_thumb.try_recv() {
                    latest.insert(name, (img, config, stock));
                }

                for (name, (base_img, mut config, stock)) in latest {
                    config.exposure_time = estimate_exposure_time(&base_img, &stock);
                    let processed = process_image(&base_img, &stock, &config);
                    let _ = tx_thumb_res.send((name, processed));
                }
                ctx.request_repaint();
            }
        });
    }
}
