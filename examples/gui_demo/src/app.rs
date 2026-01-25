use eframe::{egui, App, Frame};
use egui::{ColorImage, TextureHandle, Vec2};
use filmr::{
    estimate_exposure_time, presets, process_image, FilmMetrics, FilmStock, OutputMode,
    SimulationConfig, WhiteBalanceMode,
};
use image::imageops::FilterType;
use image::DynamicImage;
use crate::ui;

pub struct FilmrApp {
    // State
    pub original_image: Option<DynamicImage>,
    pub preview_image: Option<DynamicImage>,
    pub developed_image: Option<DynamicImage>,
    pub processed_texture: Option<TextureHandle>,
    pub original_texture: Option<TextureHandle>,
    pub metrics_original: Option<FilmMetrics>,
    pub metrics_preview: Option<FilmMetrics>,
    pub metrics_developed: Option<FilmMetrics>,

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
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let stocks = presets::get_all_stocks();
        Self {
            original_image: None,
            preview_image: None,
            developed_image: None,
            processed_texture: None,
            original_texture: None,
            metrics_original: None,
            metrics_preview: None,
            metrics_developed: None,
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

        let base_exposure = preset.r_curve.exposure_offset / 0.18;
        self.exposure_time = if let Some(img) = &self.original_image {
            estimate_exposure_time(&img.to_rgb8(), &preset)
        } else {
            base_exposure
        };
    }

    pub fn process_and_update_texture(&mut self, ctx: &egui::Context) {
        // Use preview image for GUI display to maintain responsiveness
        let source_image = self.preview_image.as_ref().or(self.original_image.as_ref());

        if let Some(img) = source_image {
            let rgb_img = img.to_rgb8();

            // Construct params
            // Use preset as base and modify
            let base_film = self.get_current_stock();

            let mut film = base_film; // Copy
            film.halation_strength = self.halation_strength;
            film.halation_threshold = self.halation_threshold;
            film.halation_sigma = self.halation_sigma;

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
            };

            // Process
            let processed = process_image(&rgb_img, &film, &config);

            // Convert to egui texture
            let size = [processed.width() as _, processed.height() as _];
            let pixels = processed.as_flat_samples();
            let color_image = ColorImage::from_rgb(size, pixels.as_slice());

            self.processed_texture = Some(ctx.load_texture(
                "processed_image",
                color_image,
                egui::TextureOptions::LINEAR,
            ));
            
            // Calculate metrics
            self.metrics_preview = Some(FilmMetrics::analyze(&processed));
        }
    }

    pub fn develop_image(&mut self, ctx: &egui::Context) {
        if let Some(img) = &self.original_image {
            self.status_msg = "Developing full resolution image...".to_owned();
            
            let rgb_img = img.to_rgb8();
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
            };

            // This can be slow, ideally run in a separate thread but for simplicity here we block
            let processed = process_image(&rgb_img, &film, &config);
            
            // Calculate developed metrics
            let developed_metrics = FilmMetrics::analyze(&processed);
            self.metrics_developed = Some(developed_metrics.clone());
            // Also update preview metrics so the panel shows the developed stats
            self.metrics_preview = Some(developed_metrics);

            // Update texture with developed result (resize for display if too large)
            let display_img = if processed.width() > 1024 || processed.height() > 1024 {
                 DynamicImage::ImageRgb8(processed.clone()).resize(1024, 1024, FilterType::Triangle).to_rgb8()
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
                            self.preview_image = Some(preview);
                            self.metrics_developed = None;
                            self.developed_image = None;
                            
                            // Load original texture (from preview)
                            if let Some(img) = &self.preview_image {
                                let rgb_img = img.to_rgb8();
                                let size = [rgb_img.width() as _, rgb_img.height() as _];
                                let pixels = rgb_img.as_flat_samples();
                                let color_image = ColorImage::from_rgb(size, pixels.as_slice());
                                self.original_texture = Some(ctx.load_texture(
                                    "original_image",
                                    color_image,
                                    egui::TextureOptions::LINEAR,
                                ));
                                
                                // Calculate original metrics (from full res image if possible but here using loaded image)
                                self.metrics_original = Some(FilmMetrics::analyze(&self.original_image.as_ref().unwrap().to_rgb8()));
                            }
                            
                            let preset = self.get_current_stock();
                            // Use preview for exposure estimation (fast enough and accurate enough)
                            let rgb_img = self.preview_image.as_ref().unwrap().to_rgb8();
                            self.exposure_time = estimate_exposure_time(&rgb_img, &preset);
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

        ui::render_controls(self, ctx);
        ui::render_metrics(self, ctx);
        ui::render_central_panel(self, ctx);
    }
}
