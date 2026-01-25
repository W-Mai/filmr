use eframe::{egui, App, Frame};
use egui::{ColorImage, Pos2, Rect, Sense, TextureHandle, Vec2};
use filmr::{
    estimate_exposure_time, presets, process_image, FilmStock, OutputMode, SimulationConfig,
    WhiteBalanceMode, FilmMetrics,
};
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
    processed_texture: Option<TextureHandle>,
    original_texture: Option<TextureHandle>,
    metrics: Option<FilmMetrics>,

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
}

impl FilmrApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            original_image: None,
            processed_texture: None,
            original_texture: None,
            metrics: None,
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
        if let Some(img) = &self.original_image {
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
            self.metrics = Some(FilmMetrics::analyze(&processed));
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
                            self.original_image = Some(img);
                            
                            // Load original texture
                            if let Some(img) = &self.original_image {
                                let rgb_img = img.to_rgb8();
                                let size = [rgb_img.width() as _, rgb_img.height() as _];
                                let pixels = rgb_img.as_flat_samples();
                                let color_image = ColorImage::from_rgb(size, pixels.as_slice());
                                self.original_texture = Some(ctx.load_texture(
                                    "original_image",
                                    color_image,
                                    egui::TextureOptions::LINEAR,
                                ));
                            }
                            
                            let preset = Self::get_preset_stock(self.selected_preset);
                            let rgb_img = self.original_image.as_ref().unwrap().to_rgb8();
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

                // Hold to Show Metrics
                self.show_metrics = ui.add_sized(
                    [150.0, 40.0],
                    egui::Button::new("HOLD FOR METRICS").min_size(Vec2::new(150.0, 40.0)),
                ).is_pointer_button_down_on();
            });
            ui.separator();

            // Metrics Overlay Window (if enabled)
            if self.show_metrics {
                egui::Window::new("Image Metrics")
                    .default_pos([200.0, 50.0])
                    .default_size([250.0, 300.0])
                    .show(ctx, |ui| {
                        if let Some(metrics) = &self.metrics {
                            ui.label(format!("Dynamic Range: {:.1} dB", metrics.dynamic_range));
                            ui.label(format!("Entropy: {:.2}", metrics.entropy));
                            ui.separator();
                            ui.label(format!("Mean RGB: {:.0}/{:.0}/{:.0}", metrics.mean_rgb[0], metrics.mean_rgb[1], metrics.mean_rgb[2]));
                            ui.label(format!("Std RGB: {:.1}/{:.1}/{:.1}", metrics.std_rgb[0], metrics.std_rgb[1], metrics.std_rgb[2]));
                            ui.separator();
                            ui.label(format!("CCT: {:.0} K", metrics.cct_tint.0));
                            ui.label(format!("Tint: {:.4}", metrics.cct_tint.1));
                            ui.label(format!("Saturation: {:.1}", metrics.saturation_mean));
                            ui.separator();
                            ui.label(format!("Texture (Lap): {:.1}", metrics.laplacian_variance));
                            ui.label(format!("PSD Slope: {:.2}", metrics.psd_slope));
                            ui.label(format!("Clipping: Z{:.1}% S{:.1}%", metrics.clipping_ratio[0]*100.0, metrics.clipping_ratio[1]*100.0));
                        } else {
                            ui.label("No metrics available");
                        }
                    });
            }

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
