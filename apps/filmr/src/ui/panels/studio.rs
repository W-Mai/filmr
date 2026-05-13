use crate::ui::app::FilmrApp;
use crate::ui::components::{labeled_slider, ACCENT, TEXT_DARK, TEXT_DISABLED};
use egui::{Color32, Ui};
use filmr::film::{FilmType, SegmentedCurve};

pub fn render_studio_panel(app: &mut FilmrApp, ctx: &egui::Context) {
    egui::SidePanel::right("studio_panel")
        .resizable(true)
        .default_width(350.0)
        .min_width(280.0)
        .max_width(380.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("STOCK STUDIO")
                        .strong()
                        .size(14.0)
                        .color(ACCENT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("✓ Done")
                                    .size(12.0)
                                    .strong()
                                    .color(TEXT_DARK),
                            )
                            .fill(ACCENT)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(4.0),
                        )
                        .clicked()
                    {
                        app.mode = crate::config::AppMode::Develop;
                    }
                });
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let mut changed = false;

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::CollapsingHeader::new(
                    egui::RichText::new("Basic Properties")
                        .strong()
                        .size(12.0)
                        .color(TEXT_DISABLED),
                )
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.label("Custom Stock");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        egui::ComboBox::from_id_salt("film_type")
                            .selected_text(format!("{:?}", app.studio_stock.film_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut app.studio_stock.film_type,
                                    FilmType::ColorNegative,
                                    "Color Negative",
                                );
                                ui.selectable_value(
                                    &mut app.studio_stock.film_type,
                                    FilmType::ColorSlide,
                                    "Color Slide",
                                );
                                ui.selectable_value(
                                    &mut app.studio_stock.film_type,
                                    FilmType::BwNegative,
                                    "B&W Negative",
                                );
                            });
                    });

                    if labeled_slider(ui, "ISO", &mut app.studio_stock.iso, 6.0..=3200.0, false) {
                        changed = true;
                    }
                    if labeled_slider(
                        ui,
                        "Resolution (lp/mm)",
                        &mut app.studio_stock.resolution_lp_mm,
                        10.0..=200.0,
                        false,
                    ) {
                        changed = true;
                    }
                    if labeled_slider(
                        ui,
                        "Reciprocity Beta",
                        &mut app.studio_stock.reciprocity.beta,
                        0.5..=1.5,
                        false,
                    ) {
                        changed = true;
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::CollapsingHeader::new(
                    egui::RichText::new("Characteristic Curves")
                        .strong()
                        .size(12.0)
                        .color(TEXT_DISABLED),
                )
                .show(ui, |ui| {
                    ui.label("Red Channel");
                    if render_curve_editor(ui, &mut app.studio_stock.r_curve, "r_curve") {
                        changed = true;
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label("Green Channel");
                    if render_curve_editor(ui, &mut app.studio_stock.g_curve, "g_curve") {
                        changed = true;
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label("Blue Channel");
                    if render_curve_editor(ui, &mut app.studio_stock.b_curve, "b_curve") {
                        changed = true;
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::CollapsingHeader::new(
                    egui::RichText::new("Spectral Sensitivity")
                        .strong()
                        .size(12.0)
                        .color(TEXT_DISABLED),
                )
                .show(ui, |ui| {
                    let params = &mut app.studio_stock.spectral_params;

                    ui.label("Red Sensitivity");
                    if labeled_slider(
                        ui,
                        "Peak Wavelength (nm)",
                        &mut params.r_peak,
                        580.0..=680.0,
                        false,
                    ) {
                        changed = true;
                    }
                    if labeled_slider(ui, "Width (nm)", &mut params.r_width, 10.0..=100.0, false) {
                        changed = true;
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label("Green Sensitivity");
                    if labeled_slider(
                        ui,
                        "Peak Wavelength (nm)",
                        &mut params.g_peak,
                        500.0..=580.0,
                        false,
                    ) {
                        changed = true;
                    }
                    if labeled_slider(ui, "Width (nm)", &mut params.g_width, 10.0..=100.0, false) {
                        changed = true;
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label("Blue Sensitivity");
                    if labeled_slider(
                        ui,
                        "Peak Wavelength (nm)",
                        &mut params.b_peak,
                        400.0..=500.0,
                        false,
                    ) {
                        changed = true;
                    }
                    if labeled_slider(ui, "Width (nm)", &mut params.b_width, 10.0..=100.0, false) {
                        changed = true;
                    }

                    // Spectral Plot
                    use egui_plot::{Line, Plot, PlotPoints};

                    let wavelengths: Vec<f64> = (380..=750).map(|w| w as f64).collect();

                    let make_series = |peak: f32, width: f32, color: Color32| {
                        let sigma = width / 2.35482; // FWHM to Sigma
                        let points: Vec<[f64; 2]> = wavelengths
                            .iter()
                            .map(|&w| {
                                let x = w as f32;
                                let z = (x - peak) / sigma;
                                let y = (-0.5 * z * z).exp();
                                [w, y as f64]
                            })
                            .collect();
                        Line::new("Spectrum", PlotPoints::new(points)).color(color)
                    };

                    let r_line = make_series(params.r_peak, params.r_width, Color32::RED);
                    let g_line = make_series(params.g_peak, params.g_width, Color32::GREEN);
                    let b_line = make_series(params.b_peak, params.b_width, Color32::BLUE);

                    Plot::new("spectral_plot")
                        .view_aspect(2.0)
                        .include_x(380.0)
                        .include_x(750.0)
                        .include_y(0.0)
                        .include_y(1.0)
                        .allow_drag(false)
                        .allow_zoom(false)
                        .allow_scroll(false)
                        .show(ui, |plot_ui| {
                            plot_ui.line(r_line);
                            plot_ui.line(g_line);
                            plot_ui.line(b_line);
                        });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::CollapsingHeader::new(
                    egui::RichText::new("Color Matrix")
                        .strong()
                        .size(12.0)
                        .color(TEXT_DISABLED),
                )
                .show(ui, |ui| {
                    ui.label("Color Correction Matrix");
                    egui::Grid::new("color_matrix_grid").show(ui, |ui| {
                        for r in 0..3 {
                            for c in 0..3 {
                                if ui
                                    .add(
                                        egui::DragValue::new(
                                            &mut app.studio_stock.color_matrix[r][c],
                                        )
                                        .speed(0.01),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            }
                            ui.end_row();
                        }
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::CollapsingHeader::new(
                    egui::RichText::new("Grain Model")
                        .strong()
                        .size(12.0)
                        .color(TEXT_DISABLED),
                )
                .show(ui, |ui| {
                    let grain = &mut app.studio_stock.grain_model;
                    if labeled_slider(ui, "Alpha (Strength)", &mut grain.alpha, 0.0..=1.0, false) {
                        changed = true;
                    }
                    if labeled_slider(
                        ui,
                        "Sigma Read (Base)",
                        &mut grain.sigma_read,
                        0.0..=0.1,
                        false,
                    ) {
                        changed = true;
                    }
                    if labeled_slider(ui, "Roughness", &mut grain.roughness, 0.0..=1.0, false) {
                        changed = true;
                    }
                    if labeled_slider(ui, "Blur Radius", &mut grain.blur_radius, 0.0..=5.0, false) {
                        changed = true;
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::CollapsingHeader::new(
                    egui::RichText::new("Halation")
                        .strong()
                        .size(12.0)
                        .color(TEXT_DISABLED),
                )
                .show(ui, |ui| {
                    if labeled_slider(
                        ui,
                        "Strength",
                        &mut app.studio_stock.halation_strength,
                        0.0..=1.0,
                        false,
                    ) {
                        changed = true;
                    }
                    if labeled_slider(
                        ui,
                        "Threshold",
                        &mut app.studio_stock.halation_threshold,
                        0.0..=1.0,
                        false,
                    ) {
                        changed = true;
                    }
                    if labeled_slider(
                        ui,
                        "Sigma (Spread)",
                        &mut app.studio_stock.halation_sigma,
                        0.0..=10.0,
                        false,
                    ) {
                        changed = true;
                    }

                    ui.label("Tint");
                    let mut color = [
                        app.studio_stock.halation_tint[0],
                        app.studio_stock.halation_tint[1],
                        app.studio_stock.halation_tint[2],
                    ];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        app.studio_stock.halation_tint = color;
                        changed = true;
                    }
                });
            });

            ui.add_space(20.0);

            if changed {
                // Sync back to the stock list if we are editing a linked stock
                if let Some(idx) = app.studio_stock_idx {
                    if idx < app.stocks.len() {
                        app.stocks[idx] = std::rc::Rc::from(app.studio_stock.clone());
                    }
                }

                app.has_unsaved_changes = true;
                app.process_and_update_texture(ctx);
            }
        });
}

fn render_curve_editor(ui: &mut Ui, curve: &mut SegmentedCurve, id_salt: &str) -> bool {
    let mut changed = false;

    // Visualize curve
    let points: Vec<[f64; 2]> = (0..=100)
        .map(|i| {
            let x = i as f32 / 100.0;
            let y = curve.map(x);
            [x as f64, y as f64]
        })
        .collect();

    use egui_plot::{Line, Plot, PlotPoints};
    let line = Line::new("Curve", PlotPoints::new(points));

    Plot::new(id_salt)
        .view_aspect(2.0)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .show(ui, |plot_ui| plot_ui.line(line));

    if labeled_slider(ui, "D Min", &mut curve.d_min, 0.0..=1.0, false) {
        changed = true;
    }
    if labeled_slider(ui, "D Max", &mut curve.d_max, 0.0..=4.0, false) {
        changed = true;
    }
    if labeled_slider(ui, "Gamma", &mut curve.gamma, 0.1..=5.0, false) {
        changed = true;
    }
    if labeled_slider(
        ui,
        "Exposure Offset (Speed)",
        &mut curve.exposure_offset,
        0.001..=1.0,
        false,
    ) {
        changed = true;
    }

    changed
}
