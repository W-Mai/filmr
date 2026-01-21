use eframe::{egui, App, Frame};
use egui::{ColorImage, TextureHandle};
use filmr::{process_image, FilmStock, GrainModel, OutputMode, SimulationConfig};
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

struct FilmrApp {
    // State
    original_image: Option<DynamicImage>,
    processed_texture: Option<TextureHandle>,
    
    // Parameters
    exposure_time: f32,
    grain_alpha: f32,
    grain_sigma: f32,
    gamma_boost: f32,
    output_mode: OutputMode,
    
    // Status
    status_msg: String,
}

impl FilmrApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            original_image: None,
            processed_texture: None,
            exposure_time: 1.0,
            grain_alpha: 0.05,
            grain_sigma: 0.01,
            gamma_boost: 1.0,
            output_mode: OutputMode::Positive,
            status_msg: "Drag and drop an image here to start.".to_owned(),
        }
    }

    fn process_and_update_texture(&mut self, ctx: &egui::Context) {
        if let Some(img) = &self.original_image {
            let rgb_img = img.to_rgb8();
            
            // Construct params
            let mut film = FilmStock::new_standard_daylight();
            // Apply gamma boost to all channels
            film.r_curve.gamma *= self.gamma_boost;
            film.g_curve.gamma *= self.gamma_boost;
            film.b_curve.gamma *= self.gamma_boost;
            
            let grain = GrainModel::new(self.grain_alpha, self.grain_sigma);
            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: self.grain_alpha > 0.0 || self.grain_sigma > 0.0,
                output_mode: self.output_mode,
            };
            
            // Process (this might be slow on main thread for large images, but okay for example)
            let processed = process_image(&rgb_img, &film, &grain, &config);
            
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
                if ui.add(egui::Slider::new(&mut self.exposure_time, 0.1..=10.0).text("Exposure Time")).changed() {
                    changed = true;
                }
                
                ui.label("Film Stock");
                if ui.add(egui::Slider::new(&mut self.gamma_boost, 0.5..=2.0).text("Gamma Boost")).changed() {
                    changed = true;
                }
            });

            ui.group(|ui| {
                ui.label("Grain");
                if ui.add(egui::Slider::new(&mut self.grain_alpha, 0.0..=0.5).text("Alpha (Shot Noise)")).changed() {
                    changed = true;
                }
                if ui.add(egui::Slider::new(&mut self.grain_sigma, 0.0..=0.2).text("Sigma (Read Noise)")).changed() {
                    changed = true;
                }
            });
            
            ui.group(|ui| {
                ui.label("Output");
                ui.horizontal(|ui| {
                    if ui.radio_value(&mut self.output_mode, OutputMode::Positive, "Positive").changed() {
                        changed = true;
                    }
                    if ui.radio_value(&mut self.output_mode, OutputMode::Negative, "Negative").changed() {
                        changed = true;
                    }
                });
            });
            
            ui.separator();
            ui.label(&self.status_msg);

            if changed {
                self.process_and_update_texture(ctx);
            }
        });

        // Main Central Panel for Image
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.processed_texture {
                // Show image scaled to fit
                ui.image((texture.id(), texture.size_vec2()));
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Drag and drop an image file here");
                });
            }
        });
    }
}
