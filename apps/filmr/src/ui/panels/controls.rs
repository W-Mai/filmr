#[cfg(not(target_arch = "wasm32"))]
use filmr::film::FilmStockCollection;
use filmr::light_leak::{LightLeak, LightLeakShape};
use filmr::{OutputMode, WhiteBalanceMode};

use egui::{Context, RichText};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use crate::config::UxMode;
use crate::ui::app::{AppMode, FilmrApp};
use egui_uix::components::toggle::Toggle;

pub struct ShutterSpeed(f64);

impl Default for ShutterSpeed {
    fn default() -> Self {
        Self(1.0 / 125.0)
    }
}

impl ShutterSpeed {
    #[allow(clippy::eq_op)]
    const STOPS: &[f64] = &[
        1.0 / 8000.0,
        1.0 / 4000.0,
        1.0 / 2000.0,
        1.0 / 1000.0,
        1.0 / 500.0,
        1.0 / 250.0,
        1.0 / 125.0,
        1.0 / 60.0,
        1.0 / 30.0,
        1.0 / 15.0,
        1.0 / 8.0,
        1.0 / 4.0,
        1.0 / 2.0,
        1.0,
        1.0 + 1.0 / 3.0,
        1.0 + 2.0 / 3.0,
        2.0,
        2.0 + 1.0 / 3.0,
        2.0 + 2.0 / 3.0,
        2.0 + 3.0 / 3.0,
        2.0 + 4.0 / 3.0,
        2.0 + 4.0 / 3.0,
        2.0 + 5.0 / 3.0,
        4.0,
        4.0 + 1.0 / 3.0,
        4.0 + 2.0 / 3.0,
        4.0 + 3.0 / 3.0,
        4.0 + 4.0 / 3.0,
        4.0 + 5.0 / 3.0,
        4.0 + 6.0 / 3.0,
        4.0 + 7.0 / 3.0,
        4.0 + 8.0 / 3.0,
        4.0 + 9.0 / 3.0,
        4.0 + 10.0 / 3.0,
        4.0 + 11.0 / 3.0,
        8.0,
        8.0 + 1.0 / 3.0,
        8.0 + 2.0 / 3.0,
        8.0 + 3.0 / 3.0,
        8.0 + 4.0 / 3.0,
        8.0 + 5.0 / 3.0,
        8.0 + 6.0 / 3.0,
        8.0 + 7.0 / 3.0,
        8.0 + 8.0 / 3.0,
        8.0 + 9.0 / 3.0,
        8.0 + 9.0 / 3.0,
        8.0 + 10.0 / 3.0,
        8.0 + 11.0 / 3.0,
        8.0 + 12.0 / 3.0,
        8.0 + 13.0 / 3.0,
        8.0 + 14.0 / 3.0,
        8.0 + 15.0 / 3.0,
        8.0 + 16.0 / 3.0,
        8.0 + 17.0 / 3.0,
        8.0 + 18.0 / 3.0,
        8.0 + 19.0 / 3.0,
        8.0 + 20.0 / 3.0,
        15.0,
        20.0,
        25.0,
        30.0,
    ];

    fn idx(&self) -> usize {
        Self::STOPS
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (*a - self.0)
                    .abs()
                    .partial_cmp(&(*b - self.0).abs())
                    .unwrap()
            })
            .map(|(i, _)| i)
            .unwrap()
    }

    pub fn display(&self) -> String {
        if self.0 < 1.0 {
            format!("1/{}", (1.0 / self.0).round())
        } else {
            format!("{:.1}\"", self.0)
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut idx = self.idx() as f64;

        let resp = ui
            .horizontal(|ui| {
                let slider = egui::Slider::new(&mut idx, 0.0..=(Self::STOPS.len() - 1) as f64)
                    .step_by(1.0)
                    .show_value(false)
                    .trailing_fill(true);

                let resp = ui.add(slider);
                ui.label(RichText::new(self.display()).size(18.0).monospace());
                resp
            })
            .inner;

        if resp.changed() {
            self.0 = Self::STOPS[(idx.round() as usize).clamp(0, Self::STOPS.len() - 1)];
        }
        resp
    }
}

pub fn render_controls(app: &mut FilmrApp, ctx: &Context) {
    let mut changed = false;
    egui::SidePanel::left("controls_panel").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                ui.heading(RichText::new("FILMR").strong().size(24.0));
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
            });
            ui.add_space(16.0);

            if app.ux_mode == UxMode::Simple {
                render_simple_controls(app, ui, ctx, &mut changed);
            } else {
                render_professional_controls(app, ui, ctx, &mut changed);
            }

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
        });

        egui::TopBottomPanel::bottom("ux_mode_panel").show(ctx, |ui| {
            // UX Mode Switcher
            ui.horizontal_centered(|ui| {
                ui.set_min_height(24.0);
                let mut toggle_flag = app.ux_mode == UxMode::Professional;

                ui.label("👶");
                if ui
                    .add(Toggle::new(&mut toggle_flag, "🚀 Professional"))
                    .clicked()
                {
                    if let Some(cm) = &mut app.config_manager {
                        cm.config.ux_mode = app.ux_mode;
                        cm.save();
                    }
                }

                app.ux_mode = if toggle_flag {
                    UxMode::Professional
                } else {
                    UxMode::Simple
                };

                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("🔧").clicked() {
                            app.show_settings = true;
                        }
                    },
                );
            });
        });
    });

    if changed {
        app.process_and_update_texture(ctx);
        app.regenerate_thumbnails();
    }
}

fn render_simple_controls(
    app: &mut FilmrApp,
    ui: &mut egui::Ui,
    _ctx: &Context,
    changed: &mut bool,
) {
    // 1. Preset Selection
    ui.label(RichText::new("🎞 Film Stock").strong());
    ui.add_space(5.0);

    let mut preset_changed = false;
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() * 0.6)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_size(ui.available_size());

                        // Group stocks by brand (first word)
                        let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
                            Default::default();
                        for (idx, stock) in app.stocks.iter().enumerate() {
                            let name = stock.full_name();
                            let brand = name
                                .split_whitespace()
                                .next()
                                .unwrap_or("Other")
                                .to_string();
                            groups.entry(brand).or_default().push(idx);
                        }

                        for (brand, indices) in groups {
                            egui::CollapsingHeader::new(
                                egui::RichText::new(brand).monospace().size(14.0),
                            )
                            .default_open(true)
                            .show(ui, |ui| {
                                for idx in indices {
                                    let stock = &app.stocks[idx];
                                    let full_name = &stock.full_name();
                                    let name = &stock.name;
                                    ui.horizontal(|ui| {
                                        if let Some(thumb) = app.preset_thumbnails.get(full_name) {
                                            let aspect =
                                                thumb.size()[0] as f32 / thumb.size()[1] as f32;
                                            let h = 40.0f32;
                                            let w = h * aspect;
                                            ui.image((thumb.id(), egui::vec2(w, h)));
                                        } else {
                                            let (rect, _) = ui.allocate_exact_size(
                                                egui::vec2(40.0, 40.0),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().rect_filled(
                                                rect,
                                                4.0,
                                                egui::Color32::from_gray(60),
                                            );
                                        }

                                        if ui
                                            .selectable_label(
                                                app.selected_stock_idx == idx,
                                                egui::RichText::new(name).monospace().size(12.0),
                                            )
                                            .clicked()
                                        {
                                            app.selected_stock_idx = idx;
                                            preset_changed = true;
                                        }
                                    });
                                    ui.add_space(2.0);
                                }
                            });
                        }
                    });
                });
        });

    if preset_changed {
        app.load_preset_values();
        *changed = true;
    }

    ui.add_space(15.0);

    // 2. Basic Adjustments
    ui.label(RichText::new("🎨 Quick Adjust").strong());
    ui.add_space(5.0);

    ui.group(|ui| {
        ui.set_min_width(ui.available_width());

        egui::Grid::new("quick_adjust_grid")
            .num_columns(2)
            .spacing(egui::vec2(10.0, 5.0))
            .show(ui, |ui| {
                // Exposure -> Brightness
                ui.label("☀ Brightness");
                if ui
                    .add(egui::Slider::new(&mut app.exposure_time, 0.001..=30.0).logarithmic(true))
                    .changed()
                {
                    *changed = true;
                }
                ui.end_row();

                // Gamma -> Contrast
                ui.label("◑ Contrast");
                if ui
                    .add(egui::Slider::new(&mut app.gamma_boost, 0.5..=1.5))
                    .changed()
                {
                    *changed = true;
                }

                ui.end_row();

                // Warmth
                ui.label("🔥 Warmth");
                if ui
                    .add(egui::Slider::new(&mut app.warmth, -1.0..=1.0))
                    .changed()
                {
                    *changed = true;
                }
                ui.end_row();

                // Saturation
                ui.label("🌈 Intensity");
                if ui
                    .add(egui::Slider::new(&mut app.saturation, 0.0..=2.0))
                    .changed()
                {
                    *changed = true;
                }

                ui.end_row();
            });
    });

    ui.add_space(15.0);
    if ui
        .button(RichText::new("✨ Auto Enhance").strong())
        .clicked()
    {
        app.white_balance_mode = WhiteBalanceMode::Auto;
        app.white_balance_strength = 1.0;
        *changed = true;
    }
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

        ui.horizontal(|ui| {
            ui.label("Exposure Time");
            let mut shutter = ShutterSpeed(app.exposure_time as f64);

            if shutter.ui(ui).changed() {
                *changed = true;
            }

            app.exposure_time = shutter.0 as f32;
        });
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
                && ui.button("📝 Edit in Studio").clicked()
            {
                app.studio_stock = app.stocks[app.selected_stock_idx].as_ref().clone();
                app.studio_stock_idx = Some(app.selected_stock_idx);
                app.mode = AppMode::StockStudio;

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
                    for (idx, stock) in app.stocks.iter().enumerate() {
                        let name = stock.full_name();
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
                                let name = app.stocks[idx].full_name();
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
                        egui::RichText::new("🛠 Custom Studio Stock")
                            .strong()
                            .color(egui::Color32::LIGHT_BLUE),
                    );
                    ui.label("Editing in Right Panel 👉");
                });
                ui.add_space(5.0);
                let mut name = app
                    .stocks
                    .get(app.studio_stock_idx.unwrap_or_default())
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                ui.label("Name:");
                egui::TextEdit::singleline(&mut name).show(ui);

                // FIXME: support name modified
                // app.stocks[app.studio_stock_idx.unwrap_or_default()].name = name;

                ui.vertical_centered(|ui| {
                    if ui
                        .button(
                            egui::RichText::new("🛠 Save & Back")
                                .strong()
                                .color(egui::Color32::LIGHT_BLUE),
                        )
                        .clicked()
                    {
                        app.mode = AppMode::Develop;
                    };
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

        let pre_om = app.output_mode;
        ui.label(RichText::new("⚙ Technical").strong());
        ui.horizontal(|ui| {
            ui.radio_value(&mut app.output_mode, OutputMode::Positive, "Positive");
            ui.radio_value(&mut app.output_mode, OutputMode::Negative, "Negative");
        });
        ui.add_space(5.0);

        let pre_wb = app.white_balance_mode;
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

        let wb_s_changed = if app.white_balance_mode != WhiteBalanceMode::Off {
            ui.add(
                egui::Slider::new(&mut app.white_balance_strength, 0.0..=1.0).text("WB Strength"),
            )
            .changed()
        } else {
            false
        };

        if pre_om != app.output_mode || pre_wb != app.white_balance_mode || wb_s_changed {
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
                for (name, mut stock) in collection.stocks {
                    if stock.name.is_empty() {
                        stock.name = name;
                    }
                    app.stocks.push(std::rc::Rc::from(stock));
                }
                app.status_msg = "Loaded preset collection".to_string();
                *changed = true;
            } else if let Ok(mut stock) = filmr::FilmStock::load_from_file(&path) {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                if stock.name.is_empty() {
                    stock.name = name.clone();
                }
                app.stocks.push(std::rc::Rc::from(stock));
                app.selected_stock_idx = app.stocks.len() - 1;
                app.load_preset_values();
                *changed = true;
                app.status_msg = format!("Loaded preset: {}", name);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn export_preset(app: &mut FilmrApp) {
    if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).save_file() {
        let mut stock = app.get_current_stock().as_ref().clone();
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
    let current_stock = app.get_current_stock().as_ref().clone();
    let base_name = app.stocks[app.selected_stock_idx].full_name();
    let clean_name = base_name.strip_prefix("Custom - ").unwrap_or(&base_name);
    let new_name = format!("Custom - {}", clean_name);
    let mut new_stock = current_stock;
    new_stock.name = new_name;
    new_stock.manufacturer = "".to_string();
    app.stocks.push(std::rc::Rc::from(new_stock.clone()));
    let new_idx = app.stocks.len() - 1;
    app.selected_stock_idx = new_idx;
    app.studio_stock = new_stock;
    app.studio_stock_idx = Some(new_idx);
    app.mode = AppMode::StockStudio;
    app.has_unsaved_changes = true;
    app.process_and_update_texture(ctx);
}
