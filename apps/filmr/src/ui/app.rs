use crate::ui::panels;
use crate::config::ConfigManager;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::{egui, App, Frame};
use egui::{ColorImage, TextureHandle, Vec2};
use filmr::film::FilmStockCollection;
use filmr::{
    estimate_exposure_time, light_leak::LightLeakConfig, presets, process_image, FilmMetrics,
    FilmStock, OutputMode, SimulationConfig, WhiteBalanceMode,
};
use image::imageops::FilterType;
use image::{DynamicImage, RgbImage};
use std::path::PathBuf;
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

struct LoadRequest {
    path: PathBuf,
}

struct LoadResult {
    path: PathBuf,
    image: Result<DynamicImage, String>,
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

    // Async Loading
    tx_load: Sender<LoadRequest>,
    rx_load: Receiver<LoadResult>,
    pub is_loading: bool,

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

    // App Mode
    pub mode: AppMode,
    pub studio_stock: FilmStock,
    pub builtin_stock_count: usize,

    // Studio State
    pub studio_stock_idx: Option<usize>,
    pub has_unsaved_changes: bool,
    pub show_exit_dialog: bool,
    pub show_settings: bool,

    pub config_manager: Option<ConfigManager>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppMode {
    Standard,
    Studio,
}

impl FilmrApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "ark-pixel".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../statics/ark-pixel-12px-monospaced-zh_cn.otf"
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

        let mut stocks = presets::get_all_stocks();
        let builtin_stock_count = stocks.len();
        let config_manager = ConfigManager::init();

        // Load custom stocks
        if let Some(cm) = &config_manager {
             if let Ok(entries) = std::fs::read_dir(&cm.config.custom_stocks_path) {
                 for entry in entries.flatten() {
                     let path = entry.path();
                     if path.extension().is_some_and(|ext| ext == "json") {
                         // Try collection first
                         if let Ok(file) = std::fs::File::open(&path) {
                             let reader = std::io::BufReader::new(file);
                             if let Ok(collection) = serde_json::from_reader::<_, FilmStockCollection>(reader) {
                                 for (name, stock) in collection.stocks {
                                      let leaked_name: &'static str = Box::leak(name.into_boxed_str());
                                      stocks.push((leaked_name, stock));
                                 }
                             } else if let Ok(stock) = FilmStock::load_from_file(&path) {
                                 let name = path.file_stem().unwrap().to_string_lossy().to_string();
                                 let leaked_name: &'static str = Box::leak(name.into_boxed_str());
                                 stocks.push((leaked_name, stock));
                             }
                         }
                     }
                 }
             }
        }

        let (tx_req, rx_req) = unbounded::<ProcessRequest>();
        let (tx_res, rx_res) = unbounded::<ProcessResult>();
        let (tx_load, rx_load) = unbounded::<LoadRequest>();
        let (tx_load_res, rx_load_res) = unbounded::<LoadResult>();

        // Clone context for the thread
        let ctx_process = cc.egui_ctx.clone();
        let ctx_load = cc.egui_ctx.clone();

        // Spawn worker thread for processing
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
                ctx_process.request_repaint();
            }
        });

        // Spawn worker thread for loading
        thread::spawn(move || {
            while let Ok(req) = rx_load.recv() {
                let res = match image::open(&req.path) {
                    Ok(img) => LoadResult {
                        path: req.path,
                        image: Ok(img),
                    },
                    Err(e) => LoadResult {
                        path: req.path,
                        image: Err(e.to_string()),
                    },
                };
                let _ = tx_load_res.send(res);
                ctx_load.request_repaint();
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

            tx_load,
            rx_load: rx_load_res,
            is_loading: false,

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

            mode: AppMode::Standard,
            studio_stock: presets::STANDARD_DAYLIGHT,
            builtin_stock_count,

            studio_stock_idx: None,
            has_unsaved_changes: false,
            show_exit_dialog: false,
            show_settings: false,

            config_manager,
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
            let base_film = if self.mode == AppMode::Studio {
                self.studio_stock
            } else {
                self.get_current_stock()
            };

            let mut film = base_film; // Copy
            if self.mode == AppMode::Standard {
                // Only apply UI overrides in Standard mode
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
            }

            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: true,
                output_mode: self.output_mode,
                white_balance_mode: self.white_balance_mode,
                white_balance_strength: self.white_balance_strength,
                light_leak: self.light_leak_config.clone(),
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

            let base_film = if self.mode == AppMode::Studio {
                self.studio_stock
            } else {
                self.get_current_stock()
            };
            let mut film = base_film;

            if self.mode == AppMode::Standard {
                film.halation_strength = self.halation_strength;
                film.halation_threshold = self.halation_threshold;
                film.halation_sigma = self.halation_sigma;
                film.r_curve.gamma *= self.gamma_boost;
                film.g_curve.gamma *= self.gamma_boost;
                film.b_curve.gamma *= self.gamma_boost;
            }

            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: true,
                output_mode: self.output_mode,
                white_balance_mode: self.white_balance_mode,
                white_balance_strength: self.white_balance_strength,
                light_leak: self.light_leak_config.clone(),
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
    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Handle File Drops
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped_files.first() {
                if let Some(path) = &file.path {
                    self.status_msg = format!("Loading {:?}...", path);
                    self.is_loading = true;
                    let _ = self.tx_load.send(LoadRequest {
                        path: path.to_path_buf(),
                    });
                }
            }
        }

        // Handle File Loading Results
        if let Ok(result) = self.rx_load.try_recv() {
            self.is_loading = false;
            match result.image {
                Ok(img) => {
                    self.original_image = Some(img.clone());
                    self.status_msg = format!("Loaded {:?}", result.path);

                    // Create original texture
                    let rgb = img.to_rgb8();
                    let size = [rgb.width() as _, rgb.height() as _];
                    let pixels = rgb.as_flat_samples();
                    let color_image = ColorImage::from_rgb(size, pixels.as_slice());
                    self.original_texture = Some(ctx.load_texture(
                        "original",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    ));
                    self.metrics_original = Some(FilmMetrics::analyze(&rgb));

                    // Generate preview
                    // Resize for performance, ensuring high quality for both landscape and portrait
                    let preview = img.resize(2048, 2048, FilterType::Lanczos3).to_rgb8();
                    self.preview_image = Some(Arc::new(preview));

                    // Initially show the raw preview image (unprocessed)
                    // This matches the requirement: "Show scaled photo initially"
                    let preview_rgb = self.preview_image.as_ref().unwrap();
                    let p_size = [preview_rgb.width() as _, preview_rgb.height() as _];
                    let p_pixels = preview_rgb.as_flat_samples();
                    let p_color_image = ColorImage::from_rgb(p_size, p_pixels.as_slice());
                    self.processed_texture = Some(ctx.load_texture(
                        "preview_raw",
                        p_color_image,
                        egui::TextureOptions::LINEAR,
                    ));

                    if self.mode == AppMode::Standard {
                        // Estimate exposure for the loaded image if in standard mode
                        let stock = self.get_current_stock();
                        self.exposure_time =
                            estimate_exposure_time(self.preview_image.as_ref().unwrap(), &stock);
                    }
                }
                Err(e) => {
                    self.status_msg = format!("Failed to load image: {}", e);
                }
            }
        }

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
                self.is_processing = false;
            } else {
                let img = result.image;
                // Convert to egui texture for display (Full Resolution)
                let size = [img.width() as _, img.height() as _];
                let pixels = img.as_flat_samples();
                let color_image = ColorImage::from_rgb(size, pixels.as_slice());

                self.processed_texture = Some(ctx.load_texture(
                    "developed_image",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));

                self.developed_image = Some(DynamicImage::ImageRgb8(img));
                self.metrics_developed = Some(result.metrics);
                self.is_processing = false;
                self.status_msg = "Development complete.".to_owned();
            }
        }

        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("Open Image...").shortcut_text("Ctrl+O"))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "tif", "tiff"])
                            .pick_file()
                        {
                            self.status_msg = format!("Loading {:?}...", path);
                            self.is_loading = true;
                            let _ = self.tx_load.send(LoadRequest { path });
                        }
                        ui.close();
                    }

                    if ui.button("Save Image...").clicked() {
                        self.save_image();
                        ui.close();
                    }

                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.separator();

                // Mode Switcher
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    if ui
                        .selectable_value(&mut self.mode, AppMode::Standard, "Standard")
                        .clicked()
                    {
                        self.process_and_update_texture(ctx);
                    }
                    if ui
                        .add_enabled(
                            self.studio_stock_idx.is_some(),
                            egui::SelectableLabel::new(self.mode == AppMode::Studio, "Stock Studio"),
                        )
                        .clicked()
                    {
                        self.mode = AppMode::Studio;
                        self.process_and_update_texture(ctx);
                    }
                });
            });
        });

        // Handle Exit Dialog
        if ctx.input(|i| i.viewport().close_requested()) && self.has_unsaved_changes {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_exit_dialog = true;
        }

        if self.show_exit_dialog {
            egui::Window::new("💾 Unsaved Custom Stock")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("You have created a custom film stock that hasn't been exported.");
                    ui.label("Are you sure you want to quit?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Quit Anyway").clicked() {
                            self.has_unsaved_changes = false;
                            self.show_exit_dialog = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_exit_dialog = false;
                        }
                    });
                });
        }

        // Left Panel (Controls) - Always show, but content adapts
        panels::controls::render_controls(self, ctx);

        // Right Panel
        if self.mode == AppMode::Studio {
            panels::studio::render_studio_panel(self, ctx);
        }

        if self.show_metrics {
            panels::metrics::render_metrics(self, ctx);
        }

        if self.show_settings {
            panels::settings::render_settings_window(self, ctx);
        }

        // Status Bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
            });
        });

        panels::central::render_central_panel(self, ctx);
    }
}
