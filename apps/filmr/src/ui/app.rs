pub use crate::config::{AppMode, ConfigManager, UxMode};
use crate::ui::panels;
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
#[cfg(not(target_arch = "wasm32"))]
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
    path: Option<PathBuf>,
    bytes: Option<Arc<[u8]>>,
    stock: Option<FilmStock>,
}

struct LoadResultData {
    image: DynamicImage,
    texture_data: ColorImage,
    metrics: FilmMetrics,
    preview: Arc<RgbImage>,
    preview_texture_data: ColorImage,
    estimated_exposure: Option<f32>,
}

struct LoadResult {
    path: Option<PathBuf>,
    result: Result<LoadResultData, String>,
}

fn process_worker_logic(req: ProcessRequest) -> ProcessResult {
    let processed = process_image(&req.image, &req.film, &req.config);
    let metrics = FilmMetrics::analyze(&processed);
    ProcessResult {
        image: processed,
        metrics,
        is_preview: req.is_preview,
    }
}

fn load_worker_logic(req: LoadRequest) -> LoadResult {
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
    pub ux_mode: UxMode,
    pub studio_stock: FilmStock,
    pub builtin_stock_count: usize,

    // Studio State
    pub studio_stock_idx: Option<usize>,
    pub has_unsaved_changes: bool,
    pub show_exit_dialog: bool,
    pub show_settings: bool,

    pub config_manager: Option<ConfigManager>,

    pub preset_thumbnails: std::collections::HashMap<&'static str, TextureHandle>,
    tx_thumb: Sender<(&'static str, RgbImage)>,
    rx_thumb: Receiver<(&'static str, RgbImage)>,

    #[cfg(target_arch = "wasm32")]
    rx_req_wasm: Receiver<ProcessRequest>,
    #[cfg(target_arch = "wasm32")]
    rx_load_wasm: Receiver<LoadRequest>,
    #[cfg(target_arch = "wasm32")]
    tx_res_wasm: Sender<ProcessResult>,
    #[cfg(target_arch = "wasm32")]
    tx_load_res_wasm: Sender<LoadResult>,

    // Preset Loading (WASM)
    #[cfg(target_arch = "wasm32")]
    pub tx_preset: Sender<Vec<u8>>,
    #[cfg(target_arch = "wasm32")]
    pub rx_preset: Receiver<Vec<u8>>,
}

impl FilmrApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "ark-pixel".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../static/ark-pixel-12px-monospaced-zh_cn.otf"
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

        // Global Style & Visuals Optimization Prepared for simple mode
        //
        // let mut style = (*cc.egui_ctx.style()).clone();
        // style.spacing.item_spacing = egui::vec2(8.0, 10.0); // Increase vertical spacing
        // style.spacing.button_padding = egui::vec2(8.0, 4.0);
        //
        // // Adjust Font Hierarchy
        // use egui::TextStyle::*;
        // style
        //     .text_styles
        //     .insert(Heading, egui::FontId::proportional(20.0));
        // style
        //     .text_styles
        //     .insert(Body, egui::FontId::proportional(14.0));
        // style
        //     .text_styles
        //     .insert(Monospace, egui::FontId::monospace(14.0));
        // style
        //     .text_styles
        //     .insert(Button, egui::FontId::proportional(14.0));
        // style
        //     .text_styles
        //     .insert(Small, egui::FontId::proportional(11.0));
        //
        // cc.egui_ctx.set_style(style);

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
                            if let Ok(collection) =
                                serde_json::from_reader::<_, FilmStockCollection>(reader)
                            {
                                for (name, stock) in collection.stocks {
                                    let leaked_name: &'static str =
                                        Box::leak(name.into_boxed_str());
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
        let (tx_thumb, rx_thumb) = unbounded::<(&'static str, RgbImage)>();
        let (tx_thumb_res, rx_thumb_res) = unbounded::<(&'static str, RgbImage)>();

        // Clone context for the thread
        let ctx_process = cc.egui_ctx.clone();
        let ctx_load = cc.egui_ctx.clone();
        let ctx_thumb = cc.egui_ctx.clone();

        // Spawn worker thread for processing
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            while let Ok(mut req) = rx_req.recv() {
                while let Ok(newer) = rx_req.try_recv() {
                    req = newer;
                }
                let res = process_worker_logic(req);
                let _ = tx_res.send(res);
                ctx_process.request_repaint();
            }
        });

        // Spawn worker thread for loading
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            while let Ok(req) = rx_load.recv() {
                let res = load_worker_logic(req);
                let _ = tx_load_res.send(res);
                ctx_load.request_repaint();
            }
        });

        // Spawn worker thread for thumbnails
        #[cfg(not(target_arch = "wasm32"))]
        let stocks_for_thumb = presets::get_all_stocks();
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            while let Ok((name, base_img)) = rx_thumb.recv() {
                // Find the stock
                if let Some(stock) = stocks_for_thumb
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, s)| s)
                {
                    let config = SimulationConfig::default();
                    let processed = process_image(&base_img, stock, &config);
                    let _ = tx_thumb_res.send((name, processed));
                    ctx_thumb.request_repaint();
                }
            }
        });

        #[cfg(target_arch = "wasm32")]
        let (tx_preset, rx_preset) = unbounded();

        let ux_mode = config_manager
            .as_ref()
            .map(|cm| cm.config.ux_mode)
            .unwrap_or(UxMode::Professional);

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

            mode: AppMode::Develop,
            ux_mode,
            studio_stock: presets::STANDARD_DAYLIGHT(),
            builtin_stock_count,

            studio_stock_idx: None,
            has_unsaved_changes: false,
            show_exit_dialog: false,
            show_settings: false,

            config_manager,

            #[cfg(target_arch = "wasm32")]
            rx_req_wasm: rx_req,
            #[cfg(target_arch = "wasm32")]
            rx_load_wasm: rx_load,
            #[cfg(target_arch = "wasm32")]
            tx_res_wasm: tx_res,
            #[cfg(target_arch = "wasm32")]
            tx_load_res_wasm: tx_load_res,

            #[cfg(target_arch = "wasm32")]
            tx_preset,
            #[cfg(target_arch = "wasm32")]
            rx_preset,
        }
    }

    pub fn get_current_stock(&self) -> FilmStock {
        if self.selected_stock_idx < self.stocks.len() {
            self.stocks[self.selected_stock_idx].1
        } else {
            presets::STANDARD_DAYLIGHT()
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
            let base_film = if self.mode == AppMode::StockStudio {
                self.studio_stock
            } else {
                self.get_current_stock()
            };

            let mut film = base_film; // Copy
            if self.mode == AppMode::Develop {
                // Only apply UI overrides in Develop mode
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
                warmth: self.warmth,
                saturation: self.saturation,
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

            let base_film = if self.mode == AppMode::StockStudio {
                self.studio_stock
            } else {
                self.get_current_stock()
            };
            let mut film = base_film;

            if self.mode == AppMode::Develop {
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
                warmth: self.warmth,
                saturation: self.saturation,
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
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(img) = &self.developed_image {
                let mut bytes: Vec<u8> = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut bytes);
                // Default to PNG
                if let Err(e) = img.write_to(&mut cursor, image::ImageOutputFormat::Png) {
                    self.status_msg = format!("Failed to encode image: {}", e);
                    return;
                }

                let task = rfd::AsyncFileDialog::new()
                    .set_file_name("filmr_output.png")
                    .save_file();

                let bytes = bytes.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(handle) = task.await {
                        if let Err(_e) = handle.write(&bytes).await {
                            // Log error?
                        }
                    }
                });
                self.status_msg = "Download started...".to_owned();
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
                let path = file.path.clone();
                let bytes = file.bytes.clone();

                if path.is_some() || bytes.is_some() {
                    let path_str = path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "dropped file".to_owned());

                    self.status_msg = format!("Loading {}...", path_str);
                    self.is_loading = true;
                    let stock = if self.mode == AppMode::Develop {
                        Some(self.get_current_stock())
                    } else {
                        None
                    };
                    let _ = self.tx_load.send(LoadRequest { path, bytes, stock });
                }
            }
        }

        // Handle File Loading Results
        if let Ok(result) = self.rx_load.try_recv() {
            self.is_loading = false;
            match result.result {
                Ok(data) => {
                    self.original_image = Some(data.image);
                    self.status_msg = format!("Loaded {:?}", result.path);

                    // Create original texture
                    self.original_texture = Some(ctx.load_texture(
                        "original",
                        data.texture_data,
                        egui::TextureOptions::LINEAR,
                    ));
                    self.metrics_original = Some(data.metrics);

                    // Reset developed status on new image load
                    self.developed_image = None;

                    // Generate preview
                    self.preview_image = Some(data.preview);

                    // Initially show the raw preview image (unprocessed)
                    // This matches the requirement: "Show scaled photo initially"
                    self.processed_texture = Some(ctx.load_texture(
                        "preview_raw",
                        data.preview_texture_data,
                        egui::TextureOptions::LINEAR,
                    ));

                    if self.mode == AppMode::Develop {
                        // Estimate exposure for the loaded image if in Develop mode
                        if let Some(exposure) = data.estimated_exposure {
                            self.exposure_time = exposure;
                        } else {
                            let stock = self.get_current_stock();
                            self.exposure_time = estimate_exposure_time(
                                self.preview_image.as_ref().unwrap(),
                                &stock,
                            );
                        }
                    }

                    // Auto-process logic: Immediately process the preview after loading
                    self.process_and_update_texture(ctx);

                    // Trigger thumbnail generation
                    let thumb_base = self
                        .original_image
                        .as_ref()
                        .unwrap()
                        .thumbnail(128, 128)
                        .to_rgb8();
                    for (name, _) in &self.stocks {
                        let _ = self.tx_thumb.send((*name, thumb_base.clone()));
                    }
                }
                Err(e) => {
                    self.status_msg = format!("Failed to load image: {}", e);
                }
            }
        }

        // Check for async results
        #[cfg(target_arch = "wasm32")]
        if let Ok(bytes) = self.rx_preset.try_recv() {
            if let Ok(collection) = serde_json::from_slice::<FilmStockCollection>(&bytes) {
                for (name, stock) in collection.stocks {
                    let leaked_name: &'static str = Box::leak(name.into_boxed_str());
                    self.stocks.push((leaked_name, stock));
                }
                self.status_msg = "Loaded preset collection".to_string();
            } else if let Ok(stock) = serde_json::from_slice::<FilmStock>(&bytes) {
                let name = format!("Imported Stock {}", self.stocks.len());
                let leaked_name: &'static str = Box::leak(name.into_boxed_str());
                self.stocks.push((leaked_name, stock));
                self.selected_stock_idx = self.stocks.len() - 1;
                self.load_preset_values();
                self.status_msg = "Loaded imported preset".to_string();
            } else {
                self.status_msg = "Failed to parse preset file".to_string();
            }
        }

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

        // Handle Thumbnail Results
        while let Ok((name, img)) = self.rx_thumb.try_recv() {
            let size = [img.width() as _, img.height() as _];
            let pixels = img.as_flat_samples();
            let color_image = ColorImage::from_rgb(size, pixels.as_slice());
            let texture = ctx.load_texture(
                format!("thumb_{}", name),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            self.preset_thumbnails.insert(name, texture);
        }

        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("Open Image...").shortcut_text("Ctrl+O"))
                        .clicked()
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "tif", "tiff"])
                            .pick_file()
                        {
                            self.status_msg = format!("Loading {:?}...", path);
                            self.is_loading = true;
                            let stock = if self.mode == AppMode::Develop {
                                Some(self.get_current_stock())
                            } else {
                                None
                            };
                            let _ = self.tx_load.send(LoadRequest {
                                path: Some(path),
                                bytes: None,
                                stock,
                            });
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            self.status_msg = "Drag and drop files to open".to_owned();
                        }
                        ui.close();
                    }

                    if ui.button("Save Image...").clicked() {
                        self.save_image();
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Settings...").clicked() {
                        self.show_settings = true;
                        ui.close();
                    }

                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.separator();

                // UX Mode Switcher
                ui.horizontal(|ui| {
                    ui.label("UX:");
                    if ui
                        .selectable_value(&mut self.ux_mode, UxMode::Simple, "🎨 Simple")
                        .clicked()
                    {
                        // Switch to Develop mode if we were in Stock Studio when switching to Simple
                        if self.mode == AppMode::StockStudio {
                            self.mode = AppMode::Develop;
                        }
                        // Save config
                        if let Some(cm) = &mut self.config_manager {
                            cm.config.ux_mode = self.ux_mode;
                            cm.save();
                        }
                    }
                    if ui
                        .selectable_value(&mut self.ux_mode, UxMode::Professional, "🚀 Pro")
                        .clicked()
                    {
                        // Save config
                        if let Some(cm) = &mut self.config_manager {
                            cm.config.ux_mode = self.ux_mode;
                            cm.save();
                        }
                    }
                });

                ui.separator();

                // Functional Mode Switcher (Only in Pro Mode)
                if self.ux_mode == UxMode::Professional {
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        if ui
                            .selectable_value(&mut self.mode, AppMode::Develop, "Develop")
                            .clicked()
                        {
                            self.process_and_update_texture(ctx);
                        }
                        if ui
                            .add_enabled(
                                self.studio_stock_idx.is_some(),
                                egui::SelectableLabel::new(
                                    self.mode == AppMode::StockStudio,
                                    "Stock Studio",
                                ),
                            )
                            .clicked()
                        {
                            self.mode = AppMode::StockStudio;
                            self.process_and_update_texture(ctx);
                        }
                    });
                }
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
        if self.mode == AppMode::StockStudio {
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

        #[cfg(target_arch = "wasm32")]
        self.poll_wasm_workers();
    }
}

#[cfg(target_arch = "wasm32")]
impl FilmrApp {
    fn poll_wasm_workers(&mut self) {
        // Process requests
        while let Ok(mut req) = self.rx_req_wasm.try_recv() {
            // Drain debounce
            while let Ok(newer) = self.rx_req_wasm.try_recv() {
                req = newer;
            }
            let res = process_worker_logic(req);
            let _ = self.tx_res_wasm.send(res);
        }

        // Load requests
        while let Ok(req) = self.rx_load_wasm.try_recv() {
            let res = load_worker_logic(req);
            let _ = self.tx_load_res_wasm.send(res);
        }
    }
}
