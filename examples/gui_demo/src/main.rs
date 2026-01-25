use eframe::{egui, App, Frame};
use egui::{ColorImage, Pos2, Rect, Sense, TextureHandle, Vec2};
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints, Points};
use filmr::{
    estimate_exposure_time, presets, process_image, FilmStock, OutputMode, SimulationConfig,
    WhiteBalanceMode, FilmMetrics,
};
use image::imageops::FilterType;
use image::DynamicImage;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Filmr GUI Example",
        options,
        Box::new(|cc| Ok(Box::new(FilmrApp::new(cc)))),
    )
}

#[derive(PartialEq, Clone, Copy)]
enum FilmPreset {
    StandardDaylight,
    // Fuji E-6
    FujifilmVelvia50,
    FujifilmVelvia100F,
    FujifilmVelvia100,
    FujifilmProvia100F,
    FujifilmAstia100F,
    FujifilmProvia400X,
    FujifilmTrebi400,
    // Fuji C-41
    FujifilmPro400H,
    FujifilmPro160NS,
    FujifilmPro160NC,
    FujifilmSuperia200,
    FujifilmSuperiaXTra800,
    // Kodak B&W
    KodakTriX400,
    KodakTMax400,
    KodakTMax100,
    KodakTMax3200,
    KodakPlusX125,
    // Ilford B&W
    IlfordHp5Plus,
    IlfordFp4Plus,
    IlfordDelta100,
    IlfordDelta400,
    IlfordPanFPlus,
    IlfordSfx200,
    // Kodak Color Negative
    KodakPortra400,
    KodakPortra160,
    KodakEktar100,
    KodakGold200,
    // Discontinued / Vintage
    Kodachrome25,
    Kodachrome64,
    KodakEktachrome100VS,
    FujifilmNeopanAcros100,
    PolaroidSx70,
}

struct FilmrApp {
    // State
    original_image: Option<DynamicImage>,
    preview_image: Option<DynamicImage>,
    developed_image: Option<DynamicImage>,
    processed_texture: Option<TextureHandle>,
    original_texture: Option<TextureHandle>,
    metrics_original: Option<FilmMetrics>,
    metrics_preview: Option<FilmMetrics>,
    metrics_developed: Option<FilmMetrics>,

    // View State
    zoom: f32,
    offset: Vec2,
    show_original: bool,
    show_metrics: bool,

    // Parameters
    exposure_time: f32,
    gamma_boost: f32,

    // Halation Parameters
    halation_strength: f32,
    halation_threshold: f32,
    halation_sigma: f32,

    // Selection
    selected_preset: FilmPreset,
    output_mode: OutputMode,
    white_balance_mode: WhiteBalanceMode,
    white_balance_strength: f32,

    // Status
    status_msg: String,

    // Metrics Display Options
    hist_log_scale: bool,
    hist_clamp_zeros: bool,
}

impl FilmrApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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

            selected_preset: FilmPreset::StandardDaylight,
            output_mode: OutputMode::Positive,
            white_balance_mode: WhiteBalanceMode::Auto,
            white_balance_strength: 1.0,
            status_msg: "Drag and drop an image here to start.".to_owned(),
            
            hist_log_scale: false,
            hist_clamp_zeros: true,
        }
    }

    fn get_preset_stock(preset: FilmPreset) -> FilmStock {
        match preset {
            FilmPreset::StandardDaylight => presets::STANDARD_DAYLIGHT,
            FilmPreset::FujifilmVelvia50 => presets::FUJIFILM_VELVIA_50,
            FilmPreset::FujifilmVelvia100F => presets::FUJIFILM_VELVIA_100F,
            FilmPreset::FujifilmVelvia100 => presets::FUJIFILM_VELVIA_100,
            FilmPreset::FujifilmProvia100F => presets::FUJIFILM_PROVIA_100F,
            FilmPreset::FujifilmAstia100F => presets::FUJIFILM_ASTIA_100F,
            FilmPreset::FujifilmProvia400X => presets::FUJIFILM_PROVIA_400X,
            FilmPreset::FujifilmTrebi400 => presets::FUJIFILM_TREBI_400,
            FilmPreset::FujifilmPro400H => presets::FUJIFILM_PRO_400H,
            FilmPreset::FujifilmPro160NS => presets::FUJIFILM_PRO_160NS,
            FilmPreset::FujifilmPro160NC => presets::FUJIFILM_PRO_160NC,
            FilmPreset::FujifilmSuperia200 => presets::FUJIFILM_SUPERIA_200,
            FilmPreset::FujifilmSuperiaXTra800 => presets::FUJIFILM_SUPERIA_X_TRA_800,
            FilmPreset::KodakTriX400 => presets::KODAK_TRI_X_400,
            FilmPreset::KodakTMax400 => presets::KODAK_T_MAX_400,
            FilmPreset::KodakTMax100 => presets::KODAK_T_MAX_100,
            FilmPreset::KodakTMax3200 => presets::KODAK_T_MAX_3200,
            FilmPreset::KodakPlusX125 => presets::KODAK_PLUS_X_125,
            FilmPreset::IlfordHp5Plus => presets::ILFORD_HP5_PLUS,
            FilmPreset::IlfordFp4Plus => presets::ILFORD_FP4_PLUS,
            FilmPreset::IlfordDelta100 => presets::ILFORD_DELTA_100,
            FilmPreset::IlfordDelta400 => presets::ILFORD_DELTA_400,
            FilmPreset::IlfordPanFPlus => presets::ILFORD_PAN_F_PLUS,
            FilmPreset::IlfordSfx200 => presets::ILFORD_SFX_200,
            FilmPreset::KodakPortra400 => presets::KODAK_PORTRA_400,
            FilmPreset::KodakPortra160 => presets::KODAK_PORTRA_160,
            FilmPreset::KodakEktar100 => presets::KODAK_EKTAR_100,
            FilmPreset::KodakGold200 => presets::KODAK_GOLD_200,
            FilmPreset::Kodachrome25 => presets::KODACHROME_25,
            FilmPreset::Kodachrome64 => presets::KODACHROME_64,
            FilmPreset::KodakEktachrome100VS => presets::KODAK_EKTACHROME_100VS,
            FilmPreset::FujifilmNeopanAcros100 => presets::FUJIFILM_NEOPAN_ACROS_100,
            FilmPreset::PolaroidSx70 => presets::POLAROID_SX_70,
        }
    }

    // Helper to load preset values into sliders when preset changes
    fn load_preset_values(&mut self) {
        let preset = Self::get_preset_stock(self.selected_preset);

        self.halation_strength = preset.halation_strength;
        self.halation_threshold = preset.halation_threshold;
        self.halation_sigma = preset.halation_sigma;

        let base_exposure = preset.r_curve.exposure_offset / 0.18;
        self.exposure_time = if let Some(img) = &self.original_image {
            estimate_exposure_time(&img.to_rgb8(), &preset)
        } else {
            base_exposure
        };

        // Grain defaults could also be tied to presets if we wanted,
        // but currently they are separate in the struct logic.
    }

    fn process_and_update_texture(&mut self, ctx: &egui::Context) {
        // Use preview image for GUI display to maintain responsiveness
        let source_image = self.preview_image.as_ref().or(self.original_image.as_ref());

        if let Some(img) = source_image {
            let rgb_img = img.to_rgb8();

            // Construct params
            // Use preset as base and modify
            let base_film = Self::get_preset_stock(self.selected_preset);

            let mut film = base_film; // Copy
            film.halation_strength = self.halation_strength;
            film.halation_threshold = self.halation_threshold;
            film.halation_sigma = self.halation_sigma;
            // film.reciprocity_beta = ... // If we had a slider for this

            // Apply gamma boost to all channels
            film.r_curve.gamma *= self.gamma_boost;
            film.g_curve.gamma *= self.gamma_boost;
            film.b_curve.gamma *= self.gamma_boost;

            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: true, // Always enable if we want grain, control via film params
                output_mode: self.output_mode,
                white_balance_mode: self.white_balance_mode,
                white_balance_strength: self.white_balance_strength,
            };

            // Process (this might be slow on main thread for large images, but okay for example)
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

    fn develop_image(&mut self, ctx: &egui::Context) {
        if let Some(img) = &self.original_image {
            self.status_msg = "Developing full resolution image...".to_owned();
            
            let rgb_img = img.to_rgb8();
            let base_film = Self::get_preset_stock(self.selected_preset);
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
            // Use same max size as preview for texture limits
            let display_img = if processed.width() > 1920 || processed.height() > 1920 {
                 DynamicImage::ImageRgb8(processed.clone()).resize(1920, 1920, FilterType::Triangle).to_rgb8()
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

    fn save_image(&mut self) {
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
                            
                            let preset = Self::get_preset_stock(self.selected_preset);
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

        // Side Panel for Controls
        egui::SidePanel::left("controls_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Filmr Controls");
                ui.separator();

                let mut changed = false;

                ui.group(|ui| {
                    ui.label("Physics");
                if ui
                    .add(
                        egui::Slider::new(&mut self.exposure_time, 0.001..=4.0)
                            .text("Exposure Time")
                            .logarithmic(true),
                    )
                    .changed()
                {
                    changed = true;
                }
                });

                ui.label("Film Stock");

                let mut preset_changed = false;

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.collapsing("Generic", |ui| {
                            if ui
                                .selectable_value(
                                    &mut self.selected_preset,
                                    FilmPreset::StandardDaylight,
                                    "Standard Daylight",
                                )
                                .clicked()
                            {
                                preset_changed = true;
                            }
                        });

                        ui.collapsing("Fujifilm (Slide)", |ui| {
                            let presets = [
                                (FilmPreset::FujifilmVelvia50, "Velvia 50"),
                                (FilmPreset::FujifilmVelvia100F, "Velvia 100F"),
                                (FilmPreset::FujifilmVelvia100, "Velvia 100"),
                                (FilmPreset::FujifilmProvia100F, "Provia 100F"),
                                (FilmPreset::FujifilmAstia100F, "Astia 100F"),
                                (FilmPreset::FujifilmProvia400X, "Provia 400X"),
                                (FilmPreset::FujifilmTrebi400, "TREBI 400"),
                            ];
                            for (p, l) in presets {
                                if ui
                                    .selectable_value(&mut self.selected_preset, p, l)
                                    .clicked()
                                {
                                    preset_changed = true;
                                }
                            }
                        });

                        ui.collapsing("Fujifilm (Color Negative)", |ui| {
                            let presets = [
                                (FilmPreset::FujifilmPro400H, "Pro 400H"),
                                (FilmPreset::FujifilmPro160NS, "Pro 160NS"),
                                (FilmPreset::FujifilmPro160NC, "Pro 160NC"),
                                (FilmPreset::FujifilmSuperia200, "Superia 200"),
                                (FilmPreset::FujifilmSuperiaXTra800, "Superia X-Tra 800"),
                            ];
                            for (p, l) in presets {
                                if ui
                                    .selectable_value(&mut self.selected_preset, p, l)
                                    .clicked()
                                {
                                    preset_changed = true;
                                }
                            }
                        });

                        ui.collapsing("Fujifilm (B&W)", |ui| {
                            if ui
                                .selectable_value(
                                    &mut self.selected_preset,
                                    FilmPreset::FujifilmNeopanAcros100,
                                    "Neopan Acros 100",
                                )
                                .clicked()
                            {
                                preset_changed = true;
                            }
                        });

                        ui.collapsing("Kodak (B&W)", |ui| {
                            let presets = [
                                (FilmPreset::KodakTriX400, "Tri-X 400"),
                                (FilmPreset::KodakTMax400, "T-Max 400"),
                                (FilmPreset::KodakTMax100, "T-Max 100"),
                                (FilmPreset::KodakTMax3200, "T-Max 3200"),
                                (FilmPreset::KodakPlusX125, "Plus-X 125"),
                            ];
                            for (p, l) in presets {
                                if ui
                                    .selectable_value(&mut self.selected_preset, p, l)
                                    .clicked()
                                {
                                    preset_changed = true;
                                }
                            }
                        });

                        ui.collapsing("Kodak (Color Negative)", |ui| {
                            let presets = [
                                (FilmPreset::KodakPortra400, "Portra 400"),
                                (FilmPreset::KodakPortra160, "Portra 160"),
                                (FilmPreset::KodakEktar100, "Ektar 100"),
                                (FilmPreset::KodakGold200, "Gold 200"),
                            ];
                            for (p, l) in presets {
                                if ui
                                    .selectable_value(&mut self.selected_preset, p, l)
                                    .clicked()
                                {
                                    preset_changed = true;
                                }
                            }
                        });

                        ui.collapsing("Kodak (Slide)", |ui| {
                            let presets = [
                                (FilmPreset::Kodachrome25, "Kodachrome 25"),
                                (FilmPreset::Kodachrome64, "Kodachrome 64"),
                                (FilmPreset::KodakEktachrome100VS, "Ektachrome 100VS"),
                            ];
                            for (p, l) in presets {
                                if ui
                                    .selectable_value(&mut self.selected_preset, p, l)
                                    .clicked()
                                {
                                    preset_changed = true;
                                }
                            }
                        });

                        ui.collapsing("Ilford (B&W)", |ui| {
                            let presets = [
                                (FilmPreset::IlfordHp5Plus, "HP5 Plus"),
                                (FilmPreset::IlfordFp4Plus, "FP4 Plus"),
                                (FilmPreset::IlfordDelta100, "Delta 100"),
                                (FilmPreset::IlfordDelta400, "Delta 400"),
                                (FilmPreset::IlfordPanFPlus, "Pan F Plus"),
                                (FilmPreset::IlfordSfx200, "SFX 200"),
                            ];
                            for (p, l) in presets {
                                if ui
                                    .selectable_value(&mut self.selected_preset, p, l)
                                    .clicked()
                                {
                                    preset_changed = true;
                                }
                            }
                        });

                        ui.collapsing("Polaroid", |ui| {
                            if ui
                                .selectable_value(
                                    &mut self.selected_preset,
                                    FilmPreset::PolaroidSx70,
                                    "SX-70",
                                )
                                .clicked()
                            {
                                preset_changed = true;
                            }
                        });

                if preset_changed {
                    self.load_preset_values();
                    changed = true;
                }

                ui.separator();

                if ui
                    .add(egui::Slider::new(&mut self.gamma_boost, 0.5..=2.0).text("Gamma Boost"))
                    .changed()
                {
                    changed = true;
                }

                ui.label("Halation");
                if ui
                    .add(
                        egui::Slider::new(&mut self.halation_strength, 0.0..=2.0)
                            .text("Strength (Glow)"),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(
                        egui::Slider::new(&mut self.halation_threshold, 0.0..=1.0)
                            .text("Threshold"),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(
                        egui::Slider::new(&mut self.halation_sigma, 0.0..=0.1)
                            .text("Sigma (Spread)"),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.group(|ui| {
                let stock = Self::get_preset_stock(self.selected_preset);
                ui.label("Grain (From Preset)");
                ui.label(format!("Alpha: {:.3}", stock.grain_model.alpha));
                ui.label(format!("Sigma: {:.3}", stock.grain_model.sigma_read));
            });

            ui.group(|ui| {
                ui.label("Output");
                ui.horizontal(|ui| {
                    if ui
                        .radio_value(&mut self.output_mode, OutputMode::Positive, "Positive")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut self.output_mode, OutputMode::Negative, "Negative")
                        .changed()
                    {
                        changed = true;
                    }
                });
            });
            ui.group(|ui| {
                ui.label("White Balance");
                ui.horizontal(|ui| {
                    if ui
                        .radio_value(&mut self.white_balance_mode, WhiteBalanceMode::Auto, "Auto")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut self.white_balance_mode, WhiteBalanceMode::Gray, "Gray")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut self.white_balance_mode,
                            WhiteBalanceMode::White,
                            "White",
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut self.white_balance_mode, WhiteBalanceMode::Off, "Off")
                        .changed()
                    {
                        changed = true;
                    }
                });
                if ui
                    .add(
                        egui::Slider::new(&mut self.white_balance_strength, 0.0..=1.0)
                            .text("Strength"),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.separator();
            ui.label(&self.status_msg);

            ui.separator();
            ui.small("Instructions:");
            ui.label("- Drag & Drop image");
            ui.label("- Scroll/Pinch to Zoom");
            ui.label("- Drag to Pan");
            ui.label("- Double Click to Reset View");

            if changed {
                self.process_and_update_texture(ctx);
            }
            }); // End ScrollArea
        });

        // Right Panel for Metrics (if enabled)
        if self.show_metrics {
            egui::SidePanel::right("metrics_panel")
                .min_width(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Image Metrics");
                    ui.separator();
                    
                    let metrics_to_show = if self.show_original {
                        &self.metrics_original
                    } else if self.developed_image.is_some() {
                        // If developed image is shown (which is handled by texture selection logic below)
                        // Actually logic below is: if show_original { original } else { processed }
                        // processed_texture is updated by both preview and develop.
                        // But develop_image updates developed_image AND processed_texture.
                        // So if developed_image is Some, we *might* be showing it.
                        // However, preview updates processed_texture too.
                        // Let's refine:
                        // If show_original -> metrics_original
                        // Else if developed_image is Some AND we haven't changed params since develop -> metrics_developed
                        // But we don't track "changed since develop".
                        // Let's assume: if developed_image is Some, we are likely looking at it OR the user is tweaking params.
                        // If user tweaks params, process_and_update_texture runs, updating processed_texture and metrics_preview.
                        // So metrics_preview is always current for processed_texture (preview).
                        // metrics_developed is for the high-res result.
                        // The texture displayed is self.processed_texture.
                        // So we should show self.metrics_preview (which matches processed_texture).
                        // EXCEPT when develop_image just ran, it updates processed_texture AND metrics_developed (we need to update metrics_preview too there or just use developed).
                        // Let's ensure develop_image updates metrics_preview too?
                        // Or simpler: always use metrics_preview for the processed view.
                        // Wait, develop_image updates metrics_developed.
                        // It also updates processed_texture.
                        // So we should probably update metrics_preview in develop_image as well?
                        // Or just display metrics_preview.
                        // Let's change develop_image to update metrics_preview too.
                        &self.metrics_preview
                    } else {
                        &self.metrics_preview
                    };

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(metrics) = metrics_to_show {
                            
                            // Helper for simple gauges
                            let gauge = |ui: &mut egui::Ui, name: &str, val: f32, min: f32, max: f32, unit: &str, color: egui::Color32| {
                                ui.horizontal(|ui| {
                                    ui.label(name);
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(format!("{:.2} {}", val, unit));
                                    });
                                });
                                let progress = ((val - min) / (max - min)).clamp(0.0, 1.0);
                                let desired_size = egui::vec2(ui.available_width(), 6.0);
                                let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
                                ui.painter().rect_filled(rect, 3.0, egui::Color32::from_gray(40));
                                let fill_width = rect.width() * progress;
                                let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
                                ui.painter().rect_filled(fill_rect, 3.0, color);
                                ui.add_space(4.0);
                            };

                            // 1. Histogram & Exposure
                            ui.collapsing("Histogram & Exposure", |ui| {
                                gauge(ui, "Dynamic Range", metrics.dynamic_range, 0.0, 15.0, "dB", egui::Color32::GOLD);
                                gauge(ui, "Entropy", metrics.entropy, 0.0, 8.0, "bits", egui::Color32::LIGHT_BLUE);
                                
                                ui.add_space(5.0);
                                ui.label("Clipping Ratio (Blacks vs Whites):");
                                let zeros = metrics.clipping_ratio[0];
                                let saturated = metrics.clipping_ratio[1];
                                Plot::new("clipping_plot")
                                    .view_aspect(6.0)
                                    .show_axes([false, false])
                                    .show_grid([false, false])
                                    .allow_zoom(false)
                                    .allow_drag(false)
                                    .allow_scroll(false)
                                    .show(ui, |plot_ui| {
                                        let bars = vec![
                                            Bar::new(0.0, zeros as f64).name("Blacks").fill(egui::Color32::RED).width(0.6),
                                            Bar::new(1.0, saturated as f64).name("Whites").fill(egui::Color32::WHITE).width(0.6),
                                        ];
                                        plot_ui.bar_chart(BarChart::new("clipping_bars", bars));
                                    });

                                ui.label("RGB Histogram:");
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut self.hist_log_scale, "Log Scale");
                                    ui.checkbox(&mut self.hist_clamp_zeros, "Ignore Blacks (0)");
                                });

                                Plot::new("rgb_hist")
                                    .view_aspect(1.5)
                                    .legend(Legend::default())
                                    .include_y(0.0)
                                    .include_y(1.05) // Leave some headroom
                                    .allow_zoom(false)
                                    .allow_drag(false)
                                    .allow_scroll(false)
                                    .show(ui, |plot_ui| {
                                        // 1. Collect all relevant bin counts to find a robust maximum (99.5th percentile)
                                        // This avoids single-bin spikes (like pure black/white) compressing the whole chart.
                                        let mut all_counts = Vec::with_capacity(256 * 3);
                                        for c in 0..3 {
                                            for (i, &v) in metrics.hist_rgb[c].iter().enumerate() {
                                                // If "Ignore Blacks" is on, skip index 0.
                                                // Also, we can optionally skip very low indices if they are just shadow noise,
                                                // but robust percentile handling should catch that naturally.
                                                if !self.hist_clamp_zeros || i > 0 {
                                                    all_counts.push(v as f64);
                                                }
                                            }
                                        }
                                        // Sort to find percentile
                                        all_counts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                                        
                                        // Get 99.5th percentile value
                                        let max_val = if all_counts.is_empty() {
                                            1.0
                                        } else {
                                            let idx = ((all_counts.len() as f64) * 0.995) as usize;
                                            all_counts[idx.min(all_counts.len() - 1)].max(1.0)
                                        };
                                        
                                        let norm_denom = if self.hist_log_scale {
                                            (max_val + 1.0).log10()
                                        } else {
                                            max_val
                                        };

                                        for (c, color) in [(0, egui::Color32::RED), (1, egui::Color32::GREEN), (2, egui::Color32::BLUE)].iter() {
                                            // Construct line points explicitly
                                            let mut line_points: Vec<[f64; 2]> = Vec::with_capacity(256);
                                            
                                            for (i, &v) in metrics.hist_rgb[*c].iter().enumerate() {
                                                if self.hist_clamp_zeros && i == 0 {
                                                    continue;
                                                }
                                                
                                                let val_raw = if self.hist_log_scale {
                                                    (v as f64 + 1.0).log10()
                                                } else {
                                                    v as f64
                                                };
                                                
                                                // Clamp to slightly above 1.0 so we see flat tops for clipped spikes
                                                let val_norm = (val_raw / norm_denom).min(1.0);
                                                line_points.push([i as f64, val_norm]);
                                            }

                                            // Draw Line on top
                                            if !line_points.is_empty() {
                                                plot_ui.line(Line::new(format!("hist_{}", c), PlotPoints::new(line_points))
                                                    .color(*color)
                                                    .name(match c { 0 => "Red", 1 => "Green", _ => "Blue" }));
                                            }
                                        }
                                    });
                            });

                            ui.separator();

                            // 2. Color Analysis
                            ui.collapsing("Color Analysis", |ui| {
                                gauge(ui, "CCT (Temp)", metrics.cct_tint.0, 2000.0, 12000.0, "K", egui::Color32::from_rgb(255, 200, 150));
                                
                                // Tint needs centered gauge
                                ui.horizontal(|ui| {
                                    ui.label("Tint");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(format!("{:.4}", metrics.cct_tint.1));
                                    });
                                });
                                let tint_val = metrics.cct_tint.1;
                                let tint_range = 0.1; // +/- 0.1
                                let tint_norm = (tint_val / tint_range).clamp(-1.0, 1.0); // -1 to 1
                                let desired_size = egui::vec2(ui.available_width(), 6.0);
                                let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
                                ui.painter().rect_filled(rect, 3.0, egui::Color32::from_gray(40));
                                let center_x = rect.center().x;
                                let bar_width = (tint_norm * rect.width() / 2.0).abs();
                                let bar_rect = if tint_norm < 0.0 {
                                    egui::Rect::from_min_size(egui::Pos2::new(center_x - bar_width, rect.min.y), egui::vec2(bar_width, rect.height()))
                                } else {
                                    egui::Rect::from_min_size(egui::Pos2::new(center_x, rect.min.y), egui::vec2(bar_width, rect.height()))
                                };
                                let tint_color = if tint_norm < 0.0 { egui::Color32::GREEN } else { egui::Color32::from_rgb(255, 0, 255) };
                                ui.painter().rect_filled(bar_rect, 3.0, tint_color);
                                ui.add_space(4.0);

                                gauge(ui, "Saturation", metrics.saturation_mean, 0.0, 100.0, "", egui::Color32::from_rgb(200, 50, 200));
                                ui.horizontal(|ui| {
                                    ui.label(format!("Sat Skew: {:.2}", metrics.saturation_skew));
                                    ui.add_space(10.0);
                                    ui.label(format!("R/G: {:.2}", metrics.rg_ratio));
                                    ui.label(format!("B/G: {:.2}", metrics.bg_ratio));
                                });
                                
                                ui.label("Lab Color Space (a* vs b*):");
                                Plot::new("lab_plot")
                                    .view_aspect(1.0)
                                    .data_aspect(1.0)
                                    .include_x(-60.0).include_x(60.0)
                                    .include_y(-60.0).include_y(60.0)
                                    .show(ui, |plot_ui| {
                                        let points = PlotPoints::from(vec![[metrics.lab_mean[1] as f64, metrics.lab_mean[2] as f64]]);
                                        plot_ui.points(Points::new("mean_lab", points).radius(6.0).shape(egui_plot::MarkerShape::Circle).color(egui::Color32::WHITE).name("Mean Color"));
                                        
                                        // Draw axes
                                        plot_ui.line(Line::new("axis_x", PlotPoints::from(vec![[-128.0, 0.0], [128.0, 0.0]])).color(egui::Color32::DARK_GRAY));
                                        plot_ui.line(Line::new("axis_y", PlotPoints::from(vec![[0.0, -128.0], [0.0, 128.0]])).color(egui::Color32::DARK_GRAY));
                                    });
                            });

                            ui.separator();

                            // 3. Texture & Structure
                            ui.collapsing("Texture & Grain", |ui| {
                                gauge(ui, "Laplacian Var", metrics.laplacian_variance, 0.0, 1000.0, "", egui::Color32::LIGHT_GRAY);
                                gauge(ui, "PSD Slope (Beta)", metrics.psd_slope, 0.0, 4.0, "", egui::Color32::YELLOW);
                                
                                ui.label("LBP Histogram (Texture Pattern):");
                                Plot::new("lbp_hist")
                                    .view_aspect(2.0)
                                    .allow_zoom(false)
                                    .allow_drag(false)
                                    .allow_scroll(false)
                                    .show(ui, |plot_ui| {
                                        let bars: Vec<Bar> = metrics.lbp_hist.iter().enumerate().map(|(i, &v)| {
                                            Bar::new(i as f64, v as f64).fill(egui::Color32::LIGHT_BLUE).width(0.8)
                                        }).collect();
                                        plot_ui.bar_chart(BarChart::new("lbp_bars", bars));
                                    });
                                    
                                ui.label("GLCM (Co-occurrence):");
                                ui.horizontal(|ui| {
                                    ui.label(format!("Contrast: {:.2}", metrics.glcm_stats[0]));
                                });
                                gauge(ui, "Correlation", metrics.glcm_stats[1], -1.0, 1.0, "", egui::Color32::LIGHT_BLUE);
                                gauge(ui, "Energy", metrics.glcm_stats[2], 0.0, 1.0, "", egui::Color32::GOLD);
                                gauge(ui, "Homogeneity", metrics.glcm_stats[3], 0.0, 1.0, "", egui::Color32::GREEN);
                            });

                            ui.separator();

                            // 4. Statistics
                            ui.collapsing("RGB Statistics", |ui| {
                                Plot::new("rgb_stats")
                                    .view_aspect(1.5)
                                    .legend(Legend::default())
                                    .allow_zoom(false)
                                    .allow_drag(false)
                                    .allow_scroll(false)
                                    .show(ui, |plot_ui| {
                                        let means = metrics.mean_rgb;
                                        let stds = metrics.std_rgb;
                                        
                                        let bars = vec![
                                            Bar::new(0.0, means[0] as f64).name("Red").fill(egui::Color32::RED).width(0.4),
                                            Bar::new(1.0, means[1] as f64).name("Green").fill(egui::Color32::GREEN).width(0.4),
                                            Bar::new(2.0, means[2] as f64).name("Blue").fill(egui::Color32::BLUE).width(0.4),
                                        ];
                                        plot_ui.bar_chart(BarChart::new("rgb_means", bars));
                                        
                                        // Error bars
                                        for i in 0..3 {
                                            let x = i as f64;
                                            let y = means[i] as f64;
                                            let s = stds[i] as f64;
                                            plot_ui.line(Line::new(format!("err_v_{}", i), PlotPoints::from(vec![[x, y-s], [x, y+s]])).color(egui::Color32::WHITE).name(""));
                                            plot_ui.line(Line::new(format!("err_t_{}", i), PlotPoints::from(vec![[x-0.1, y-s], [x+0.1, y-s]])).color(egui::Color32::WHITE).name(""));
                                            plot_ui.line(Line::new(format!("err_b_{}", i), PlotPoints::from(vec![[x-0.1, y+s], [x+0.1, y+s]])).color(egui::Color32::WHITE).name(""));
                                        }
                                    });
                                    
                                ui.label("Skewness & Kurtosis:");
                                Plot::new("skew_kurt")
                                    .view_aspect(2.0)
                                    .legend(Legend::default())
                                    .include_y(-5.0)
                                    .include_y(5.0)
                                    .allow_zoom(false)
                                    .allow_drag(false)
                                    .allow_scroll(false)
                                    .show(ui, |plot_ui| {
                                        let mut bars_skew = Vec::new();
                                        let mut bars_kurt = Vec::new();
                                        
                                        for i in 0..3 {
                                             let color = match i { 0 => egui::Color32::RED, 1 => egui::Color32::GREEN, _ => egui::Color32::BLUE };
                                             bars_skew.push(Bar::new(i as f64, metrics.skewness_rgb[i] as f64).fill(color).width(0.3).name(match i {0=>"R Skew", 1=>"G Skew", _=>"B Skew"}));
                                             bars_kurt.push(Bar::new((i+4) as f64, metrics.kurtosis_rgb[i] as f64).fill(color).width(0.3).name(match i {0=>"R Kurt", 1=>"G Kurt", _=>"B Kurt"}));
                                        }
                                        plot_ui.bar_chart(BarChart::new("skew_bars", bars_skew));
                                        plot_ui.bar_chart(BarChart::new("kurt_bars", bars_kurt));
                                    });
                                    
                                ui.collapsing("RGB Quantiles", |ui| {
                                    egui::Grid::new("quantiles_grid").striped(true).spacing([20.0, 4.0]).show(ui, |ui| {
                                        ui.label(egui::RichText::new("Channel").strong());
                                        ui.label(egui::RichText::new("P10").strong());
                                        ui.label(egui::RichText::new("P50").strong());
                                        ui.label(egui::RichText::new("P90").strong());
                                        ui.label(egui::RichText::new("P99").strong());
                                        ui.end_row();
                                        
                                        let names = ["Red", "Green", "Blue"];
                                        for (i, name) in names.iter().enumerate() {
                                            ui.label(*name);
                                            for q in 0..4 {
                                                ui.label(format!("{}", metrics.quantiles_rgb[i][q]));
                                            }
                                            ui.end_row();
                                        }
                                    });
                                });
                            });

                        } else {
                            ui.label("No metrics available. Load an image.");
                        }
                    });
                });
        }

        // Main Central Panel for Image
        egui::CentralPanel::default().show(ctx, |ui| {
            // Toolbar Overlay
            ui.horizontal(|ui| {
                // Hold to Compare (Larger and Conspicuous)
                self.show_original = ui.add_sized(
                    [150.0, 40.0],
                    egui::Button::new("HOLD TO COMPARE").min_size(Vec2::new(150.0, 40.0)),
                ).is_pointer_button_down_on();
                
                ui.separator();

                // Toggle Metrics Panel
                ui.toggle_value(&mut self.show_metrics, "Metrics Panel");

                ui.separator();

                if ui.add_sized([100.0, 40.0], egui::Button::new("Develop")).clicked() {
                    self.develop_image(ctx);
                }

                let save_btn = egui::Button::new("Save").min_size(Vec2::new(100.0, 40.0));
                if ui.add_enabled(self.developed_image.is_some(), save_btn).clicked() {
                    self.save_image();
                }
            });
            ui.separator();

            let texture_to_show = if self.show_original {
                self.original_texture.as_ref()
            } else {
                self.processed_texture.as_ref()
            };

            if let Some(texture) = texture_to_show {
                // Interactive Area
                // We use ui.available_rect_before_wrap() to get the full area
                let rect = ui.available_rect_before_wrap();
                let response =
                    ui.interact(rect, ui.id().with("image_area"), Sense::click_and_drag());

                // 1. Handle Zoom (Pinch or Ctrl+Scroll)
                // ctx.input(|i| i.zoom_delta()) handles both pinch gestures and ctrl+scroll
                let zoom_delta = ctx.input(|i| i.zoom_delta());
                if zoom_delta != 1.0 {
                    // Zoom towards mouse pointer
                    if let Some(pointer_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        // Pointer position relative to center of screen (or current offset)
                        // Actually easier:
                        // NewOffset = Pointer - (Pointer - OldOffset) * scale_factor
                        // But here offset is "translation of image center from screen center".

                        // Let's model it as:
                        // ImagePos = ScreenCenter + Offset
                        // PointOnImage = (Pointer - ImagePos) / Zoom
                        // We want PointOnImage to stay at Pointer after Zoom changes.

                        // A standard approach:
                        // Offset -= (Pointer - Center - Offset) * (zoom_delta - 1.0)
                        let center = rect.center();
                        let pointer_in_layer = pointer_pos - center;
                        let offset_to_pointer = pointer_in_layer - self.offset;

                        self.offset -= offset_to_pointer * (zoom_delta - 1.0);
                        self.zoom *= zoom_delta;
                    } else {
                        // Just zoom center if no pointer
                        self.zoom *= zoom_delta;
                    }
                }

                // 2. Handle Pan (Drag)
                if response.dragged() {
                    self.offset += response.drag_delta();
                }

                // 3. Double Click to Reset
                if response.double_clicked() {
                    self.zoom = 1.0;
                    self.offset = Vec2::ZERO;
                }

                // 4. Draw Image
                // Calculate size and position
                let image_size = texture.size_vec2();
                // Fit to screen initially if zoom is 1.0?
                // Let's say zoom=1.0 means "fit to view" or "100%"?
                // Usually for photo viewers, initial is "fit".
                // Let's check aspect ratios.
                let aspect = image_size.x / image_size.y;
                let view_aspect = rect.width() / rect.height();

                let base_scale = if aspect > view_aspect {
                    rect.width() / image_size.x
                } else {
                    rect.height() / image_size.y
                };

                let current_scale = base_scale * self.zoom;
                let displayed_size = image_size * current_scale;

                let center = rect.center() + self.offset;
                let image_rect = Rect::from_center_size(center, displayed_size);

                // Draw
                let painter = ui.painter_at(rect);
                painter.image(
                    texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Drag and drop an image file here");
                });
            }
        });
    }
}
