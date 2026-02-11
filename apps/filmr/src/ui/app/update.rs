//! App trait implementation for FilmrApp.

use super::workers::LoadRequest;
use super::FilmrApp;
use crate::config::AppMode;
use crate::ui::panels;
use eframe::{App, Frame};
use egui::{ColorImage, Context};
use filmr::estimate_exposure_time;
#[cfg(target_arch = "wasm32")]
use filmr::film::FilmStockCollection;
#[cfg(target_arch = "wasm32")]
use filmr::FilmStock;
use image::DynamicImage;

impl App for FilmrApp {
    #[allow(deprecated)]
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
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
                        Some(self.get_current_stock().as_ref().clone())
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
                    self.source_path = result.path.clone();
                    self.status_msg = format!("Loaded {:?}", result.path);

                    // Read EXIF metadata from source file
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        self.source_exif = result
                            .path
                            .as_ref()
                            .and_then(|p| little_exif::metadata::Metadata::new_from_path(p).ok());
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.source_exif = None;
                    }

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

                    // Trigger thumbnail generation with current UI config
                    self.regenerate_thumbnails();
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
                for (name, mut stock) in collection.stocks {
                    if stock.name.is_empty() {
                        stock.name = name;
                    }
                    self.stocks.push(std::rc::Rc::from(stock));
                }
                self.status_msg = "Loaded preset collection".to_string();
            } else if let Ok(stock) = serde_json::from_slice::<FilmStock>(&bytes) {
                let name = format!("Imported Stock {}", self.stocks.len());
                let mut stock = stock;
                if stock.name.is_empty() {
                    stock.name = name;
                }
                self.stocks.push(std::rc::Rc::from(stock));
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
                self.developed_image = None;
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
    }
}
