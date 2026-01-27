use crate::ui::app::{AppMode, FilmrApp};
use egui::Context;
use filmr::light_leak::{LightLeak, LightLeakShape};
use filmr::{OutputMode, WhiteBalanceMode};

use rfd::FileDialog;

pub fn render_controls(app: &mut FilmrApp, ctx: &Context) {
    egui::SidePanel::left("controls_panel").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Filmr Controls");
            ui.separator();

            let mut changed = false;

            // Preset Management (Standard Mode Only)
            if app.mode == AppMode::Standard {
                ui.group(|ui| {
                    ui.label("Preset Management");
                    ui.horizontal(|ui| {
                        if ui.button("Import Preset").clicked() {
                            if let Some(path) =
                                FileDialog::new().add_filter("JSON", &["json"]).pick_file()
                            {
                                if let Ok(stock) = filmr::FilmStock::load_from_file(&path) {
                                    let name =
                                        path.file_stem().unwrap().to_string_lossy().to_string();

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
                                    let leaked_name: &'static str =
                                        Box::leak(name.into_boxed_str());
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

                        if ui.button("Export Preset").clicked() {
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
            }

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

            if app.mode == AppMode::Standard {
                if ui.button("✨ Create Custom Stock from Current").clicked() {
                    let current_stock = app.get_current_stock();
                    let new_stock = current_stock;

                    let base_name = app.stocks[app.selected_stock_idx].0;
                    // Extract name without "Custom - " prefix if it already exists to avoid stacking
                    let clean_name = base_name.strip_prefix("Custom - ").unwrap_or(base_name);
                    let new_name = format!("Custom - {}", clean_name);

                    let leaked_name: &'static str = Box::leak(new_name.into_boxed_str());

                    app.stocks.push((leaked_name, new_stock));
                    let new_idx = app.stocks.len() - 1;
                    app.selected_stock_idx = new_idx;

                    app.studio_stock = new_stock;
                    app.studio_stock_idx = Some(new_idx);
                    app.mode = AppMode::Studio;
                    app.has_unsaved_changes = true;

                    app.process_and_update_texture(ctx);
                }
                ui.add_space(5.0);

                let mut preset_changed = false;

                egui::ScrollArea::vertical()
                    .max_height(200.0) // Reduced height to fit other controls
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
            } else {
                // Studio Mode: Show only the temporary stock
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("🛠️ Custom Studio Stock")
                                .strong()
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                        ui.label("Editing in Right Panel 👉");
                    });
                });
            }

            ui.separator();

            // Only show overrides in Standard Mode
            if app.mode == AppMode::Standard {
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
                    .add(
                        egui::Slider::new(&mut app.halation_threshold, 0.0..=1.0).text("Threshold"),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(
                        egui::Slider::new(&mut app.halation_sigma, 0.0..=0.1)
                            .text("Sigma (Spread)"),
                    )
                    .changed()
                {
                    changed = true;
                }
            }

            ui.group(|ui| {
                ui.label("Light Leaks");
                if ui
                    .checkbox(&mut app.light_leak_config.enabled, "Enable Light Leaks")
                    .changed()
                {
                    changed = true;
                }

                if app.light_leak_config.enabled {
                    ui.horizontal(|ui| {
                        if ui.button("Add Leak").clicked() {
                            app.light_leak_config.leaks.push(LightLeak {
                                position: (0.5, 0.5),
                                color: [1.0, 0.8, 0.6], // Warm default
                                radius: 0.5,
                                intensity: 0.5,
                                shape: LightLeakShape::Circle,
                            });
                            changed = true;
                        }
                        if ui.button("Clear All").clicked() {
                            app.light_leak_config.leaks.clear();
                            changed = true;
                        }
                    });

                    let mut leaks_to_remove = Vec::new();
                    for (i, leak) in app.light_leak_config.leaks.iter_mut().enumerate() {
                        ui.collapsing(format!("Leak #{}", i + 1), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Pos:");
                                if ui
                                    .add(
                                        egui::Slider::new(&mut leak.position.0, 0.0..=1.0)
                                            .text("X"),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                if ui
                                    .add(
                                        egui::Slider::new(&mut leak.position.1, 0.0..=1.0)
                                            .text("Y"),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                if ui.color_edit_button_rgb(&mut leak.color).changed() {
                                    changed = true;
                                }
                            });

                            if ui
                                .add(egui::Slider::new(&mut leak.radius, 0.0..=1.5).text("Radius"))
                                .changed()
                            {
                                changed = true;
                            }
                            if ui
                                .add(
                                    egui::Slider::new(&mut leak.intensity, 0.0..=2.0)
                                        .text("Intensity"),
                                )
                                .changed()
                            {
                                changed = true;
                            }

                            egui::ComboBox::from_id_salt(format!("shape_{}", i))
                                .selected_text(format!("{:?}", leak.shape))
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(
                                            &mut leak.shape,
                                            LightLeakShape::Circle,
                                            "Circle",
                                        )
                                        .clicked()
                                    {
                                        changed = true;
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut leak.shape,
                                            LightLeakShape::Linear,
                                            "Linear",
                                        )
                                        .clicked()
                                    {
                                        changed = true;
                                    }
                                });

                            if ui.button("Remove").clicked() {
                                leaks_to_remove.push(i);
                                changed = true;
                            }
                        });
                    }

                    if !leaks_to_remove.is_empty() {
                        for i in leaks_to_remove.into_iter().rev() {
                            app.light_leak_config.leaks.remove(i);
                        }
                    }
                }
            });

            if app.mode == AppMode::Standard {
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
            }

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
