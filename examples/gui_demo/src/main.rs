use eframe::{egui, App, Frame};
use egui::{ColorImage, Pos2, Rect, Sense, TextureHandle, Vec2};
use filmr::{presets, process_image, FilmStock, OutputMode, SimulationConfig};
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
    KodakTriX400,
    FujifilmVelvia50,
    IlfordHp5Plus,
}

struct FilmrApp {
    // State
    original_image: Option<DynamicImage>,
    processed_texture: Option<TextureHandle>,

    // View State
    zoom: f32,
    offset: Vec2,

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

    // Status
    status_msg: String,
}

impl FilmrApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            original_image: None,
            processed_texture: None,
            zoom: 1.0,
            offset: Vec2::ZERO,
            exposure_time: 1.0,
            gamma_boost: 1.0,

            // Default Halation params
            halation_strength: 0.0,
            halation_threshold: 0.8,
            halation_sigma: 0.02,

            selected_preset: FilmPreset::StandardDaylight,
            output_mode: OutputMode::Positive,
            status_msg: "Drag and drop an image here to start.".to_owned(),
        }
    }

    // Helper to load preset values into sliders when preset changes
    fn load_preset_values(&mut self) {
        let preset = match self.selected_preset {
            FilmPreset::StandardDaylight => presets::STANDARD_DAYLIGHT,
            FilmPreset::KodakTriX400 => presets::KODAK_TRI_X_400,
            FilmPreset::FujifilmVelvia50 => presets::FUJIFILM_VELVIA_50,
            FilmPreset::IlfordHp5Plus => presets::ILFORD_HP5_PLUS,
        };

        self.halation_strength = preset.halation_strength;
        self.halation_threshold = preset.halation_threshold;
        self.halation_sigma = preset.halation_sigma;

        // Grain defaults could also be tied to presets if we wanted, 
        // but currently they are separate in the struct logic.
    }

    fn process_and_update_texture(&mut self, ctx: &egui::Context) {
        if let Some(img) = &self.original_image {
            let rgb_img = img.to_rgb8();

            // Construct params
            // Use preset as base and modify
            let base_film = match self.selected_preset {
                FilmPreset::StandardDaylight => presets::STANDARD_DAYLIGHT,
                FilmPreset::KodakTriX400 => presets::KODAK_TRI_X_400,
                FilmPreset::FujifilmVelvia50 => presets::FUJIFILM_VELVIA_50,
                FilmPreset::IlfordHp5Plus => presets::ILFORD_HP5_PLUS,
            };

            let mut film = FilmStock::new(
                base_film.iso,
                base_film.r_curve,
                base_film.g_curve,
                base_film.b_curve,
                base_film.color_matrix,
                base_film.grain_model,
                base_film.resolution_lp_mm,
                base_film.reciprocity_exponent,
                self.halation_strength,
                self.halation_threshold,
                self.halation_sigma,
                base_film.halation_tint, // Keep tint from preset for now, or add color picker later
            );

            // Apply gamma boost to all channels
            film.r_curve.gamma *= self.gamma_boost;
            film.g_curve.gamma *= self.gamma_boost;
            film.b_curve.gamma *= self.gamma_boost;

            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: true, // Always enable if we want grain, control via film params
                output_mode: self.output_mode,
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
            ui.heading("Filmr Controls");
            ui.separator();

            let mut changed = false;

            ui.group(|ui| {
                ui.label("Physics");
                if ui
                    .add(
                        egui::Slider::new(&mut self.exposure_time, 0.1..=10.0)
                            .text("Exposure Time"),
                    )
                    .changed()
                {
                    changed = true;
                }

                ui.label("Film Stock");
                
                // Preset ComboBox
                egui::ComboBox::from_label("Preset")
                    .selected_text(match self.selected_preset {
                        FilmPreset::StandardDaylight => "Standard Daylight",
                        FilmPreset::KodakTriX400 => "Kodak Tri-X 400",
                        FilmPreset::FujifilmVelvia50 => "Fujifilm Velvia 50",
                        FilmPreset::IlfordHp5Plus => "Ilford HP5 Plus",
                    })
                    .show_ui(ui, |ui| {
                        let mut preset_changed = false;
                        if ui.selectable_value(&mut self.selected_preset, FilmPreset::StandardDaylight, "Standard Daylight").clicked() { preset_changed = true; }
                        if ui.selectable_value(&mut self.selected_preset, FilmPreset::KodakTriX400, "Kodak Tri-X 400").clicked() { preset_changed = true; }
                        if ui.selectable_value(&mut self.selected_preset, FilmPreset::FujifilmVelvia50, "Fujifilm Velvia 50").clicked() { preset_changed = true; }
                        if ui.selectable_value(&mut self.selected_preset, FilmPreset::IlfordHp5Plus, "Ilford HP5 Plus").clicked() { preset_changed = true; }
                        
                        if preset_changed {
                            self.load_preset_values();
                            changed = true;
                        }
                    });

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
                ui.label("Grain (From Preset)");
                ui.label(format!("Alpha: {:.3}", match self.selected_preset {
                    FilmPreset::StandardDaylight => presets::STANDARD_DAYLIGHT.grain_model.alpha,
                    FilmPreset::KodakTriX400 => presets::KODAK_TRI_X_400.grain_model.alpha,
                    FilmPreset::FujifilmVelvia50 => presets::FUJIFILM_VELVIA_50.grain_model.alpha,
                    FilmPreset::IlfordHp5Plus => presets::ILFORD_HP5_PLUS.grain_model.alpha,
                }));
                 ui.label(format!("Sigma: {:.3}", match self.selected_preset {
                    FilmPreset::StandardDaylight => presets::STANDARD_DAYLIGHT.grain_model.sigma_read,
                    FilmPreset::KodakTriX400 => presets::KODAK_TRI_X_400.grain_model.sigma_read,
                    FilmPreset::FujifilmVelvia50 => presets::FUJIFILM_VELVIA_50.grain_model.sigma_read,
                    FilmPreset::IlfordHp5Plus => presets::ILFORD_HP5_PLUS.grain_model.sigma_read,
                }));
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
        });

        // Main Central Panel for Image
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.processed_texture {
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
