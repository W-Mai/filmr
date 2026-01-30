use crate::ui::app::{AppMode, FilmrApp};
use egui::{Context, RichText};
#[cfg(not(target_arch = "wasm32"))]
use filmr::film::FilmStockCollection;
use filmr::light_leak::{LightLeak, LightLeakShape};
use filmr::{OutputMode, WhiteBalanceMode};

#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

pub fn render_controls(app: &mut FilmrApp, ctx: &Context) {
    egui::SidePanel::left("controls_panel").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                ui.heading(RichText::new("FILMR").strong().size(24.0));
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
            });
            ui.add_space(16.0);

            let mut changed = false;

            render_professional_controls(app, ui, ctx, &mut changed);

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.collapsing("⌨ Shortcuts & Help", |ui| {
                ui.small("- Drag & Drop image to open");
                ui.small("- Scroll/Pinch to Zoom");
                ui.small("- Drag image to Pan");
                ui.small("- Double Click to Reset View");
                ui.small("- Ctrl+O to Open File");
            });

            if changed {
                app.process_and_update_texture(ctx);
            }
        });
    });
}

fn render_professional_controls(
    app: &mut FilmrApp,
    ui: &mut egui::Ui,
    ctx: &Context,
    changed: &mut bool,
) {
    // 1. Preset Management
    if app.mode == AppMode::Develop {
        ui.set_min_width(ui.available_width());
        ui.collapsing("📦 Preset Management", |ui| {
            ui.horizontal(|ui| {
                if ui.button("Import").clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    import_preset(app, changed);
                }
                if ui.button("Export").clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    export_preset(app);
                }
            });
        });
    }

    ui.add_space(5.0);

    // 2. Physics
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("🔬 Physics").strong());
        if ui
            .add(
                egui::Slider::new(&mut app.exposure_time, 0.001..=4.0)
                    .text("Exposure Time")
                    .logarithmic(true),
            )
            .changed()
        {
            *changed = true;
        }
    });

    ui.add_space(5.0);

    // 3. Film Stock
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("🎞 Film Stock").strong());

        if app.mode == AppMode::Develop {
            if ui.button("✨ Create Custom Stock").clicked() {
                create_custom_stock(app, ctx);
                app.process_and_update_texture(ctx);
            }

            // Allow editing if it is a custom stock (imported or created)
            if app.selected_stock_idx >= app.builtin_stock_count
                && ui.button("📝 Edit in Stock Studio").clicked()
            {
                app.studio_stock = app.stocks[app.selected_stock_idx].1;
                app.studio_stock_idx = Some(app.selected_stock_idx);
                app.mode = AppMode::StockStudio;
                // Editing existing stock implies potential changes
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
                *changed = true;
            }
        } else {
            // Studio Mode: Show only the temporary stock
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
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
    });

    ui.add_space(5.0);

    // 4. Look Overrides
    if app.mode == AppMode::Develop {
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new("🎨 Look Overrides").strong());
            if ui
                .add(egui::Slider::new(&mut app.gamma_boost, 0.5..=2.0).text("Gamma Boost"))
                .changed()
            {
                *changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.warmth, -1.0..=1.0).text("Warmth"))
                .changed()
            {
                *changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.saturation, 0.0..=2.0).text("Saturation"))
                .changed()
            {
                *changed = true;
            }
        });

        ui.add_space(5.0);

        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new("🏮 Halation").strong());
            if ui
                .add(
                    egui::Slider::new(&mut app.halation_strength, 0.0..=2.0)
                        .text("Strength (Glow)"),
                )
                .changed()
            {
                *changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.halation_threshold, 0.0..=1.0).text("Threshold"))
                .changed()
            {
                *changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.halation_sigma, 0.0..=0.1).text("Sigma (Spread)"))
                .changed()
            {
                *changed = true;
            }
        });

        ui.add_space(5.0);

        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new("🌾 Grain").strong());

            if ui
                .add(egui::Slider::new(&mut app.grain_alpha, 0.0..=0.05).text("Alpha"))
                .changed()
            {
                *changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.grain_sigma, 0.0..=0.05).text("Sigma"))
                .changed()
            {
                *changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.grain_roughness, 0.0..=1.0).text("Roughness"))
                .changed()
            {
                *changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut app.grain_blur_radius, 0.0..=2.0).text("Blur"))
                .changed()
            {
                *changed = true;
            }
        });
    }

    ui.add_space(5.0);

    // 5. Light Leaks
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("🔦 Light Leaks").strong());

        if ui
            .checkbox(&mut app.light_leak_config.enabled, "Enable")
            .changed()
        {
            *changed = true;
        }
        if app.light_leak_config.enabled {
            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    app.light_leak_config.leaks.push(LightLeak::default());
                    *changed = true;
                }
                if ui.button("Clear").clicked() {
                    app.light_leak_config.leaks.clear();
                    *changed = true;
                }
            });

            let mut leaks_to_remove = Vec::new();
            for (i, leak) in app.light_leak_config.leaks.iter_mut().enumerate() {
                ui.collapsing(format!("Leak #{}", i + 1), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Pos:");
                        if ui
                            .add(egui::Slider::new(&mut leak.position.0, 0.0..=1.0).text("X"))
                            .changed()
                        {
                            *changed = true;
                        }
                        if ui
                            .add(egui::Slider::new(&mut leak.position.1, 0.0..=1.0).text("Y"))
                            .changed()
                        {
                            *changed = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        if ui.color_edit_button_rgb(&mut leak.color).changed() {
                            *changed = true;
                        }
                    });

                    if ui
                        .add(egui::Slider::new(&mut leak.radius, 0.0..=1.5).text("Radius"))
                        .changed()
                    {
                        *changed = true;
                    }
                    if ui
                        .add(egui::Slider::new(&mut leak.intensity, 0.0..=2.0).text("Intensity"))
                        .changed()
                    {
                        *changed = true;
                    }

                    if ui
                        .add(
                            egui::Slider::new(&mut leak.rotation, 0.0..=std::f32::consts::TAU)
                                .text("Rotation"),
                        )
                        .changed()
                    {
                        *changed = true;
                    }

                    if ui
                        .add(egui::Slider::new(&mut leak.roughness, 0.0..=1.0).text("Roughness"))
                        .changed()
                    {
                        *changed = true;
                    }

                    egui::ComboBox::from_id_salt(format!("shape_{}", i))
                        .selected_text(format!("{:?}", leak.shape))
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(&mut leak.shape, LightLeakShape::Circle, "Circle")
                                .clicked()
                            {
                                *changed = true;
                            }
                            if ui
                                .selectable_value(&mut leak.shape, LightLeakShape::Linear, "Linear")
                                .clicked()
                            {
                                *changed = true;
                            }
                            if ui
                                .selectable_value(
                                    &mut leak.shape,
                                    LightLeakShape::Organic,
                                    "Organic",
                                )
                                .clicked()
                            {
                                *changed = true;
                            }
                            if ui
                                .selectable_value(&mut leak.shape, LightLeakShape::Plasma, "Plasma")
                                .clicked()
                            {
                                *changed = true;
                            }
                        });

                    if ui.button("Remove").clicked() {
                        leaks_to_remove.push(i);
                        *changed = true;
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

    ui.add_space(5.0);

    // 6. Technical
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("⚙ Technical").strong());
        ui.horizontal(|ui| {
            ui.radio_value(&mut app.output_mode, OutputMode::Positive, "Positive");
            ui.radio_value(&mut app.output_mode, OutputMode::Negative, "Negative");
        });
        ui.add_space(5.0);
        egui::ComboBox::from_label("White Balance")
            .selected_text(format!("{:?}", app.white_balance_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.white_balance_mode, WhiteBalanceMode::Auto, "Auto");
                ui.selectable_value(&mut app.white_balance_mode, WhiteBalanceMode::Gray, "Gray");
                ui.selectable_value(
                    &mut app.white_balance_mode,
                    WhiteBalanceMode::White,
                    "White",
                );
                ui.selectable_value(&mut app.white_balance_mode, WhiteBalanceMode::Off, "Off");
            });
        if ui
            .add(egui::Slider::new(&mut app.white_balance_strength, 0.0..=1.0).text("WB Strength"))
            .changed()
        {
            *changed = true;
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn import_preset(app: &mut FilmrApp, changed: &mut bool) {
    if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
        if let Ok(file) = std::fs::File::open(&path) {
            let reader = std::io::BufReader::new(file);
            if let Ok(collection) = serde_json::from_reader::<_, FilmStockCollection>(reader) {
                for (name, stock) in collection.stocks {
                    let leaked_name: &'static str = Box::leak(name.into_boxed_str());
                    app.stocks.push((leaked_name, stock));
                }
                app.status_msg = "Loaded preset collection".to_string();
                *changed = true;
            } else if let Ok(stock) = filmr::FilmStock::load_from_file(&path) {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let leaked_name: &'static str = Box::leak(name.into_boxed_str());
                app.stocks.push((leaked_name, stock));
                app.selected_stock_idx = app.stocks.len() - 1;
                app.load_preset_values();
                *changed = true;
                app.status_msg = format!("Loaded preset: {}", leaked_name);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn export_preset(app: &mut FilmrApp) {
    if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).save_file() {
        let mut stock = app.get_current_stock();
        stock.halation_strength = app.halation_strength;
        stock.halation_threshold = app.halation_threshold;
        stock.halation_sigma = app.halation_sigma;
        stock.grain_model.alpha = app.grain_alpha;
        stock.grain_model.sigma_read = app.grain_sigma;
        stock.grain_model.roughness = app.grain_roughness;
        stock.grain_model.blur_radius = app.grain_blur_radius;
        stock.r_curve.gamma *= app.gamma_boost;
        stock.g_curve.gamma *= app.gamma_boost;
        stock.b_curve.gamma *= app.gamma_boost;

        if stock.save_to_file(&path).is_ok() {
            app.status_msg = format!("Saved preset to {:?}", path);
        }
    }
}

fn create_custom_stock(app: &mut FilmrApp, ctx: &Context) {
    let current_stock = app.get_current_stock();
    let base_name = app.stocks[app.selected_stock_idx].0;
    let clean_name = base_name.strip_prefix("Custom - ").unwrap_or(base_name);
    let new_name = format!("Custom - {}", clean_name);
    let leaked_name: &'static str = Box::leak(new_name.into_boxed_str());

    app.stocks.push((leaked_name, current_stock));
    let new_idx = app.stocks.len() - 1;
    app.selected_stock_idx = new_idx;
    app.studio_stock = current_stock;
    app.studio_stock_idx = Some(new_idx);
    app.mode = AppMode::StockStudio;
    app.has_unsaved_changes = true;
    app.process_and_update_texture(ctx);
}
