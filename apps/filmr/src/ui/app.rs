use crate::ui::panels;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::{egui, App, Frame};
use egui::{ColorImage, TextureHandle, Vec2};
use filmr::{
    estimate_exposure_time, light_leak::LightLeakConfig, presets, process_image, FilmMetrics,
    FilmStock, OutputMode, SimulationConfig, WhiteBalanceMode,
};
use image::imageops::FilterType;
use image::{DynamicImage, RgbImage};
use std::sync::Arc;
use std::thread;

struct ProcessRequest {
    image: Arc<RgbImage>,
    film: FilmStock,
    config: SimulationConfig,
    is_preview: bool,
}

struct ProcessResult {
    image: RgbImage,
    metrics: FilmMetrics,
    is_preview: bool,
}

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

    // Async Processing
    tx_req: Sender<ProcessRequest>,
    rx_res: Receiver<ProcessResult>,
    pub is_processing: bool,

    // View State
    pub zoom: f32,
    pub offset: Vec2,
    pub show_original: bool,
    pub show_metrics: bool,

    // Parameters
    pub exposure_time: f32,
    pub gamma_boost: f32,

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
    pub stocks: Vec<(&'static str, FilmStock)>,
    pub selected_stock_idx: usize,

    pub output_mode: OutputMode,
    pub white_balance_mode: WhiteBalanceMode,
    pub white_balance_strength: f32,

    // Status
    pub status_msg: String,

    // Metrics Display Options
    pub hist_log_scale: bool,
    pub hist_clamp_zeros: bool,
}

impl FilmrApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let stocks = presets::get_all_stocks();
        let (tx_req, rx_req) = unbounded::<ProcessRequest>();
        let (tx_res, rx_res) = unbounded::<ProcessResult>();

        // Clone context for the thread
        let ctx = cc.egui_ctx.clone();

        // Spawn worker thread
        thread::spawn(move || {
            while let Ok(mut req) = rx_req.recv() {
                // Drain any newer requests to skip intermediate states (debounce)
                while let Ok(newer) = rx_req.try_recv() {
                    req = newer;
                }

                // Process
                let processed = process_image(&req.image, &req.film, &req.config);
                let metrics = FilmMetrics::analyze(&processed);

                // Send back result
                let _ = tx_res.send(ProcessResult {
                    image: processed,
                    metrics,
                    is_preview: req.is_preview,
                });

                // Wake up the GUI
                ctx.request_repaint();
            }
        });

        Self {
            original_image: None,
            preview_image: None,
            developed_image: None,
            processed_texture: None,
            original_texture: None,
            metrics_original: None,
            metrics_preview: None,
            metrics_developed: None,

            tx_req,
            rx_res,
            is_processing: false,

            zoom: 1.0,
            offset: Vec2::ZERO,
            show_original: false,
            show_metrics: false,
            exposure_time: 1.0,
            gamma_boost: 1.0,

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
        }
    }

    pub fn get_current_stock(&self) -> FilmStock {
        if self.selected_stock_idx < self.stocks.len() {
            self.stocks[self.selected_stock_idx].1
        } else {
            presets::STANDARD_DAYLIGHT
        }
    }

    // Helper to load preset values into sliders when preset changes
    pub fn load_preset_values(&mut self) {
        let preset = self.get_current_stock();

        self.halation_strength = preset.halation_strength;
        self.halation_threshold = preset.halation_threshold;
        self.halation_sigma = preset.halation_sigma;

        self.grain_alpha = preset.grain_model.alpha;
        self.grain_sigma = preset.grain_model.sigma_read;
        self.grain_roughness = preset.grain_model.roughness;
        self.grain_blur_radius = preset.grain_model.blur_radius;

        let base_exposure = preset.r_curve.exposure_offset / 0.18;
        self.exposure_time = if let Some(img) = &self.original_image {
            estimate_exposure_time(&img.to_rgb8(), &preset)
        } else {
            base_exposure
        };
    }

    pub fn process_and_update_texture(&mut self, _ctx: &egui::Context) {
        // Use preview image for GUI display to maintain responsiveness
        // For preview, we use the pre-converted Arc<RgbImage>
        let source_image = self.preview_image.as_ref();

        if let Some(img) = source_image {
            // Construct params
            // Use preset as base and modify
            let base_film = self.get_current_stock();

            let mut film = base_film; // Copy
            film.halation_strength = self.halation_strength;
            film.halation_threshold = self.halation_threshold;
            film.halation_sigma = self.halation_sigma;

            film.grain_model.alpha = self.grain_alpha;
            film.grain_model.sigma_read = self.grain_sigma;
            film.grain_model.roughness = self.grain_roughness;
            film.grain_model.blur_radius = self.grain_blur_radius;

            // Apply gamma boost to all channels
            film.r_curve.gamma *= self.gamma_boost;
            film.g_curve.gamma *= self.gamma_boost;
            film.b_curve.gamma *= self.gamma_boost;

            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: true,
                output_mode: self.output_mode,
                white_balance_mode: self.white_balance_mode,
                white_balance_strength: self.white_balance_strength,
                light_leak: self.light_leak_config.clone(),
                ..Default::default()
            };

            // Send request to worker
            // Direct clone of Arc, O(1)
            let request = ProcessRequest {
                image: Arc::clone(img),
                film,
                config,
                is_preview: true,
            };

            let _ = self.tx_req.send(request);
            self.is_processing = true;
        }
    }

    pub fn develop_image(&mut self, _ctx: &egui::Context) {
        if let Some(img) = &self.original_image {
            self.status_msg = "Developing full resolution image...".to_owned();

            // This might still take a bit of time to clone/convert, but it's unavoidable for full-res develop
            // unless we also keep full-res as RgbImage (memory intensive).
            let rgb_img = Arc::new(img.to_rgb8());

            let base_film = self.get_current_stock();
            let mut film = base_film;
            film.halation_strength = self.halation_strength;
            film.halation_threshold = self.halation_threshold;
            film.halation_sigma = self.halation_sigma;
            film.r_curve.gamma *= self.gamma_boost;
            film.g_curve.gamma *= self.gamma_boost;
            film.b_curve.gamma *= self.gamma_boost;

            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: true,
                output_mode: self.output_mode,
                white_balance_mode: self.white_balance_mode,
                white_balance_strength: self.white_balance_strength,
                light_leak: self.light_leak_config.clone(),
                ..Default::default()
            };

            let request = ProcessRequest {
                image: rgb_img,
                film,
                config,
                is_preview: false,
            };

            let _ = self.tx_req.send(request);
            self.is_processing = true;
        }
    }

    pub fn save_image(&mut self) {
        if let Some(img) = &self.developed_image {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .save_file()
            {
                if let Err(e) = img.save(&path) {
                    self.status_msg = format!("Failed to save image: {}", e);
                } else {
                    self.status_msg = format!("Saved to {:?}", path);
                }
            }
        }
    }
}

impl App for FilmrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Check for async results
        if let Ok(result) = self.rx_res.try_recv() {
            if result.is_preview {
                // Convert to egui texture
                let size = [result.image.width() as _, result.image.height() as _];
                let pixels = result.image.as_flat_samples();
                let color_image = ColorImage::from_rgb(size, pixels.as_slice());

                self.processed_texture = Some(ctx.load_texture(
                    "processed_image",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));

                self.metrics_preview = Some(result.metrics);
            } else {
                // Handle Development Result
                let processed = result.image;
                // Calculate developed metrics (metrics already calculated in worker)
                self.metrics_developed = Some(result.metrics.clone());
                // Also update preview metrics
                self.metrics_preview = Some(result.metrics);

                // Update texture with developed result (resize for display if too large)
                let display_img = if processed.width() > 1024 || processed.height() > 1024 {
                    DynamicImage::ImageRgb8(processed.clone())
                        .resize(1024, 1024, FilterType::Triangle)
                        .to_rgb8()
                } else {
                    processed.clone()
                };

                let size = [display_img.width() as _, display_img.height() as _];
                let pixels = display_img.as_flat_samples();
                let color_image = ColorImage::from_rgb(size, pixels.as_slice());

                self.processed_texture = Some(ctx.load_texture(
                    "processed_image",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));

                self.developed_image = Some(DynamicImage::ImageRgb8(processed));
                self.status_msg = "Development complete. Ready to save.".to_owned();
            }
            self.is_processing = false;
        }

        // Handle file drop
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped_files.first() {
                if let Some(path) = &file.path {
                    match image::open(path) {
                        Ok(img) => {
                            // Keep original full resolution image
                            self.original_image = Some(img.clone());

                            // Create preview for GUI (max 1024px)
                            let preview = if img.width() > 1024 || img.height() > 1024 {
                                img.resize(1024, 1024, FilterType::Triangle)
                            } else {
                                img
                            };
                            self.preview_image = Some(Arc::new(preview.to_rgb8()));
                            self.metrics_developed = None;
                            self.developed_image = None;

                            // Load original texture (from preview)
                            if let Some(rgb_img) = &self.preview_image {
                                let size = [rgb_img.width() as _, rgb_img.height() as _];
                                let pixels = rgb_img.as_flat_samples();
                                let color_image = ColorImage::from_rgb(size, pixels.as_slice());
                                self.original_texture = Some(ctx.load_texture(
                                    "original_image",
                                    color_image,
                                    egui::TextureOptions::LINEAR,
                                ));

                                // Calculate original metrics (from full res image if possible but here using loaded image)
                                self.metrics_original = Some(FilmMetrics::analyze(
                                    &self.original_image.as_ref().unwrap().to_rgb8(),
                                ));
                            }

                            let preset = self.get_current_stock();
                            // Use preview for exposure estimation (fast enough and accurate enough)
                            // preview_image is now Arc<RgbImage>
                            if let Some(rgb_img) = &self.preview_image {
                                self.exposure_time = estimate_exposure_time(rgb_img, &preset);
                            }
                            self.status_msg = format!("Loaded: {:?}", path.file_name().unwrap());
                            // Reset view
                            self.zoom = 1.0;
                            self.offset = Vec2::ZERO;
                            self.process_and_update_texture(ctx);
                        }
                        Err(err) => {
                            self.status_msg = format!("Error loading file: {}", err);
                        }
                    }
                }
            }
        }

        panels::controls::render_controls(self, ctx);
        panels::metrics::render_metrics(self, ctx);
        panels::central::render_central_panel(self, ctx);
    }
}
