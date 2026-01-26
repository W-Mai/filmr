use crate::ui::app::FilmrApp;
use egui::Context;
use filmr::{OutputMode, WhiteBalanceMode};

use rfd::FileDialog;

pub fn render_controls(app: &mut FilmrApp, ctx: &Context) {
    egui::SidePanel::left("controls_panel").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Filmr Controls");
            ui.separator();

            let mut changed = false;

            ui.group(|ui| {
                ui.label("Preset Management");
                ui.horizontal(|ui| {
                    if ui.button("Import JSON").clicked() {
                        if let Some(path) =
                            FileDialog::new().add_filter("JSON", &["json"]).pick_file()
                        {
                            if let Ok(stock) = filmr::FilmStock::load_from_file(&path) {
                                let name = path.file_stem().unwrap().to_string_lossy().to_string();

                                // Update UI sliders to match the loaded stock
                                app.halation_strength = stock.halation_strength;
                                app.halation_threshold = stock.halation_threshold;
                                app.halation_sigma = stock.halation_sigma;

                                app.grain_alpha = stock.grain_model.alpha;
                                app.grain_sigma = stock.grain_model.sigma_read;
                                app.grain_roughness = stock.grain_model.roughness;
                                app.grain_blur_radius = stock.grain_model.blur_radius;

                                // Add the loaded stock to the list and select it
                                // Note: We use Box::leak to extend the 'static lifetime for the demo
                                let leaked_name: &'static str = Box::leak(name.into_boxed_str());
                                app.stocks.push((leaked_name, stock));
                                app.selected_stock_idx = app.stocks.len() - 1;

                                app.load_preset_values();
                                changed = true;
                                app.status_msg = format!("Loaded preset: {}", leaked_name);
                            } else {
                                app.status_msg = "Failed to load preset".to_string();
                            }
                        }
                    }

                    if ui.button("Export JSON").clicked() {
                        if let Some(path) =
                            FileDialog::new().add_filter("JSON", &["json"]).save_file()
                        {
                            // Create a new stock based on current selection and UI adjustments
                            let base = app.get_current_stock();
                            let mut current_stock = base;

                            // Apply current UI parameters
                            current_stock.halation_strength = app.halation_strength;
                            current_stock.halation_threshold = app.halation_threshold;
                            current_stock.halation_sigma = app.halation_sigma;
                            current_stock.grain_model.alpha = app.grain_alpha;
                            current_stock.grain_model.sigma_read = app.grain_sigma;
                            current_stock.grain_model.roughness = app.grain_roughness;
                            current_stock.grain_model.blur_radius = app.grain_blur_radius;

                            // Apply Gamma Boost to curves for visual consistency
                            current_stock.r_curve.gamma *= app.gamma_boost;
                            current_stock.g_curve.gamma *= app.gamma_boost;
                            current_stock.b_curve.gamma *= app.gamma_boost;

                            if current_stock.save_to_file(&path).is_ok() {
                                app.status_msg = format!("Saved preset to {:?}", path);
                            } else {
                                app.status_msg = "Failed to save preset".to_string();
                            }
                        }
                    }
                });
            });

            ui.group(|ui| {
                ui.label("Physics");
                if ui
                    .add(
                        egui::Slider::new(&mut app.exposure_time, 0.001..=4.0)
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
                    // Group stocks by brand (first word)
                    let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
                        Default::default();
                    for (idx, (name, _)) in app.stocks.iter().enumerate() {
                        let brand = name
                            .split_whitespace()
                            .next()
                            .unwrap_or("Other")
                            .to_string();
                        groups.entry(brand).or_default().push(idx);
                    }

                    for (brand, indices) in groups {
                        ui.collapsing(brand, |ui| {
                            for idx in indices {
                                let (name, _) = app.stocks[idx];
                                if ui
                                    .selectable_value(&mut app.selected_stock_idx, idx, name)
                                    .clicked()
                                {
                                    preset_changed = true;
                                }
                            }
                        });
                    }
                });

            if preset_changed {
                app.load_preset_values();
                changed = true;
            }

            ui.separator();

            if ui
                .add(egui::Slider::new(&mut app.gamma_boost, 0.5..=2.0).text("Gamma Boost"))
                .changed()
            {
                changed = true;
            }

            ui.label("Halation");
            if ui
                .add(
                    egui::Slider::new(&mut app.halation_strength, 0.0..=2.0)
                        .text("Strength (Glow)"),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.halation_threshold, 0.0..=1.0).text("Threshold"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.halation_sigma, 0.0..=0.1).text("Sigma (Spread)"))
                .changed()
            {
                changed = true;
            }

            ui.group(|ui| {
                ui.label("Grain (Editable)");

                ui.label("Alpha (Intensity)");
                if ui
                    .add(egui::Slider::new(&mut app.grain_alpha, 0.0..=0.05).step_by(0.001))
                    .changed()
                {
                    changed = true;
                }

                ui.label("Sigma (Base Noise)");
                if ui
                    .add(egui::Slider::new(&mut app.grain_sigma, 0.0..=0.05).step_by(0.001))
                    .changed()
                {
                    changed = true;
                }

                ui.label("Roughness");
                if ui
                    .add(egui::Slider::new(&mut app.grain_roughness, 0.0..=1.0))
                    .changed()
                {
                    changed = true;
                }

                ui.label("Blur Radius");
                if ui
                    .add(egui::Slider::new(&mut app.grain_blur_radius, 0.0..=2.0))
                    .changed()
                {
                    changed = true;
                }
            });

            ui.group(|ui| {
                ui.label("Output");
                ui.horizontal(|ui| {
                    if ui
                        .radio_value(&mut app.output_mode, OutputMode::Positive, "Positive")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut app.output_mode, OutputMode::Negative, "Negative")
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
                        .radio_value(&mut app.white_balance_mode, WhiteBalanceMode::Auto, "Auto")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut app.white_balance_mode, WhiteBalanceMode::Gray, "Gray")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut app.white_balance_mode,
                            WhiteBalanceMode::White,
                            "White",
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut app.white_balance_mode, WhiteBalanceMode::Off, "Off")
                        .changed()
                    {
                        changed = true;
                    }
                });
                if ui
                    .add(
                        egui::Slider::new(&mut app.white_balance_strength, 0.0..=1.0)
                            .text("Strength"),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.separator();
            ui.label(&app.status_msg);

            ui.separator();
            ui.small("Instructions:");
            ui.label("- Drag & Drop image");
            ui.label("- Scroll/Pinch to Zoom");
            ui.label("- Drag to Pan");
            ui.label("- Double Click to Reset View");

            if changed {
                app.process_and_update_texture(ctx);
            }
        });
    });
}
