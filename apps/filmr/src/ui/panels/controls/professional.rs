use egui::{Context, RichText};
use filmr::light_leak::{LightLeak, LightLeakShape};
use filmr::{FilmStyle, OutputMode, WhiteBalanceMode};

use crate::ui::app::{AppMode, FilmrApp};

use super::preset_io::create_custom_stock;
use super::shutter_speed::ShutterSpeed;

#[cfg(not(target_arch = "wasm32"))]
use super::preset_io::{export_preset, import_preset};

pub fn render_professional_controls(
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
    render_film_stock_section(app, ui, ctx, changed);

    ui.add_space(5.0);

    // 4. Rendering Style
    if app.mode == AppMode::Develop {
        render_rendering_style(app, ui, changed);
        ui.add_space(5.0);
    }

    // 5. Look Overrides
    if app.mode == AppMode::Develop {
        render_look_overrides(app, ui, changed);
        ui.add_space(5.0);
        render_halation(app, ui, changed);
        ui.add_space(5.0);
        render_grain(app, ui, changed);
    }

    ui.add_space(5.0);

    // 6. Light Leaks
    render_light_leaks(app, ui, changed);

    ui.add_space(5.0);

    // 7. Technical
    render_technical(app, ui, changed);
}

fn render_film_stock_section(
    app: &mut FilmrApp,
    ui: &mut egui::Ui,
    ctx: &Context,
    changed: &mut bool,
) {
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
                .max_height(200.0)
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
}

fn render_look_overrides(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
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
        if ui
            .add(egui::Slider::new(&mut app.motion_blur_amount, 0.0..=3.0).text("Motion Blur"))
            .changed()
        {
            *changed = true;
        }

        // Trajectory preview
        if app.motion_blur_amount > 0.0 {
            ui.horizontal(|ui| {
                if ui.button("🎲").on_hover_text("New trajectory").clicked() {
                    app.motion_blur_seed = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;
                    *changed = true;
                }
                ui.label(format!("seed: {}", app.motion_blur_seed));
            });

            let traj = filmr::shake::ShakeTrajectory::generate(
                app.motion_blur_amount * 25.0,
                64,
                app.motion_blur_seed,
            );

            ui.horizontal(|ui| {
                draw_trajectory_canvas(ui, &traj);
                draw_dwell_chart(ui, &traj);
            });
        }
    });
}

fn draw_trajectory_canvas(ui: &mut egui::Ui, traj: &filmr::shake::ShakeTrajectory) {
    let size = egui::vec2(120.0, 120.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    let cx = rect.center().x;
    let cy = rect.center().y;
    let pts: Vec<egui::Pos2> = traj
        .points
        .iter()
        .map(|&(x, y, _)| egui::pos2(cx + x, cy + y))
        .collect();

    if pts.len() >= 2 {
        for (i, w) in pts.windows(2).enumerate() {
            let weight = traj.points[i].2;
            let alpha = (weight * traj.points.len() as f32 * 255.0).clamp(30.0, 255.0) as u8;
            let stroke = egui::Stroke::new(
                1.5,
                egui::Color32::from_rgba_unmultiplied(255, 120, 60, alpha),
            );
            painter.line_segment([w[0], w[1]], stroke);
        }
        painter.circle_filled(pts[0], 3.0, egui::Color32::from_rgb(80, 220, 80));
        painter.circle_filled(
            *pts.last().unwrap(),
            3.0,
            egui::Color32::from_rgb(220, 80, 80),
        );
    }
}

fn draw_dwell_chart(ui: &mut egui::Ui, traj: &filmr::shake::ShakeTrajectory) {
    let size = egui::vec2(120.0, 120.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    let n = traj.points.len();
    if n <= 1 {
        return;
    }

    let max_w = traj.points.iter().map(|p| p.2).fold(0.0f32, f32::max);
    let bar_w = rect.width() / n as f32;

    for (i, &(_, _, w)) in traj.points.iter().enumerate() {
        let h = if max_w > 0.0 {
            w / max_w * (rect.height() - 4.0)
        } else {
            0.0
        };
        let x = rect.left() + i as f32 * bar_w;
        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(x, rect.bottom() - h),
            egui::vec2(bar_w.max(1.0), h),
        );
        let t = w / max_w.max(1e-6);
        let r = (t * 255.0) as u8;
        let b = ((1.0 - t) * 200.0) as u8;
        painter.rect_filled(bar_rect, 0.0, egui::Color32::from_rgb(r, 40, b));
    }

    painter.text(
        egui::pos2(rect.left() + 2.0, rect.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "dwell",
        egui::FontId::proportional(9.0),
        egui::Color32::from_gray(150),
    );
}

fn render_halation(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("🏮 Halation").strong());
        if ui
            .add(egui::Slider::new(&mut app.halation_strength, 0.0..=2.0).text("Strength (Glow)"))
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
}

fn render_rendering_style(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("🎨 Rendering Style").strong());

        let prev_style = app.film_style;
        ui.horizontal_wrapped(|ui| {
            for style in FilmStyle::all() {
                ui.selectable_value(&mut app.film_style, style, style.name());
            }
        });

        if app.film_style != prev_style {
            *changed = true;
        }

        ui.small(app.film_style.description());
    });
}

fn render_grain(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
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

fn render_light_leaks(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
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
}

fn render_technical(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
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
