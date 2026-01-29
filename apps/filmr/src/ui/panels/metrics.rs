use crate::ui::app::{FilmrApp, UxMode};
use egui::Context;
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints, Points};

pub fn render_metrics(app: &mut FilmrApp, ctx: &Context) {
    if app.show_metrics {
        egui::SidePanel::right("metrics_panel")
            .min_width(350.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Image Metrics");
                ui.separator();

                let metrics_to_show = if app.show_original {
                    &app.metrics_original
                } else if app.developed_image.is_some() {
                    &app.metrics_developed
                } else {
                    &app.metrics_preview
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(metrics) = metrics_to_show {
                        // 1. Histogram (Always show)
                        render_rgb_histogram(
                            ui,
                            metrics,
                            &mut app.hist_log_scale,
                            &mut app.hist_clamp_zeros,
                            app.ux_mode,
                        );

                        // 2. Advanced Metrics (Only in Pro Mode)
                        if app.ux_mode == UxMode::Professional {
                            render_advanced_metrics(ui, metrics);
                        }
                    } else {
                        ui.label("No metrics available. Load an image.");
                    }
                });
            });
    }
}

fn render_rgb_histogram(
    ui: &mut egui::Ui,
    metrics: &filmr::FilmMetrics,
    hist_log_scale: &mut bool,
    hist_clamp_zeros: &mut bool,
    ux_mode: UxMode,
) {
    let mut plot_hist = |ui: &mut egui::Ui| {
        ui.horizontal(|ui| {
            ui.checkbox(hist_log_scale, "Log Scale");
            ui.checkbox(hist_clamp_zeros, "Ignore Blacks");
        });
        Plot::new("rgb_hist")
            .view_aspect(1.5)
            .legend(Legend::default())
            .include_y(0.0)
            .include_y(1.05)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                let mut all_counts = Vec::with_capacity(256 * 3);
                for c in 0..3 {
                    for (i, &v) in metrics.hist_rgb[c].iter().enumerate() {
                        if !*hist_clamp_zeros || i > 0 {
                            all_counts.push(v as f64);
                        }
                    }
                }
                all_counts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let max_val = if all_counts.is_empty() {
                    1.0
                } else {
                    let idx = ((all_counts.len() as f64) * 0.995) as usize;
                    all_counts[idx.min(all_counts.len() - 1)].max(1.0)
                };

                let norm_denom = if *hist_log_scale {
                    (max_val + 1.0).log10()
                } else {
                    max_val
                };

                for (c, color) in [
                    (0, egui::Color32::RED),
                    (1, egui::Color32::GREEN),
                    (2, egui::Color32::BLUE),
                ] {
                    let mut line_points: Vec<[f64; 2]> = Vec::with_capacity(256);
                    for (i, &v) in metrics.hist_rgb[c].iter().enumerate() {
                        if *hist_clamp_zeros && i == 0 {
                            continue;
                        }
                        let val_raw = if *hist_log_scale {
                            (v as f64 + 1.0).log10()
                        } else {
                            v as f64
                        };
                        let val_norm = (val_raw / norm_denom).min(1.0);
                        line_points.push([i as f64, val_norm]);
                    }
                    if !line_points.is_empty() {
                        plot_ui.line(
                            Line::new(format!("hist_{}", c), PlotPoints::new(line_points))
                                .color(color)
                                .name(match c {
                                    0 => "Red",
                                    1 => "Green",
                                    _ => "Blue",
                                }),
                        );
                    }
                }
            });
    };

    if ux_mode == UxMode::Simple {
        plot_hist(ui);
    } else {
        ui.collapsing("RGB Histogram", |ui| {
            plot_hist(ui);
        });
    }
}

fn render_advanced_metrics(ui: &mut egui::Ui, metrics: &filmr::FilmMetrics) {
    ui.separator();

    // Helper for simple gauges
    let gauge = |ui: &mut egui::Ui,
                 name: &str,
                 val: f32,
                 min: f32,
                 max: f32,
                 unit: &str,
                 color: egui::Color32| {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{:.2} {}", val, unit));
            });
        });
        let progress = ((val - min) / (max - min)).clamp(0.0, 1.0);
        let desired_size = egui::vec2(ui.available_width(), 6.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_gray(40));
        let fill_width = rect.width() * progress;
        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
        ui.painter().rect_filled(fill_rect, 3.0, color);
        ui.add_space(4.0);
    };

    ui.collapsing("Exposure & Range", |ui| {
        gauge(
            ui,
            "Dynamic Range",
            metrics.dynamic_range,
            0.0,
            15.0,
            "dB",
            egui::Color32::GOLD,
        );
        gauge(
            ui,
            "Entropy",
            metrics.entropy,
            0.0,
            8.0,
            "bits",
            egui::Color32::LIGHT_BLUE,
        );
        ui.add_space(5.0);
        ui.label("Clipping Ratio (Blacks vs Whites):");
        let zeros = metrics.clipping_ratio[0];
        let saturated = metrics.clipping_ratio[1];
        Plot::new("clipping_plot")
            .view_aspect(6.0)
            .show_axes([false, false])
            .show_grid([false, false])
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                let bars = vec![
                    Bar::new(0.0, zeros as f64)
                        .name("Blacks")
                        .fill(egui::Color32::RED)
                        .width(0.6),
                    Bar::new(1.0, saturated as f64)
                        .name("Whites")
                        .fill(egui::Color32::WHITE)
                        .width(0.6),
                ];
                plot_ui.bar_chart(BarChart::new("clipping_bars", bars));
            });
    });

    ui.separator();

    ui.collapsing("Color Analysis", |ui| {
        gauge(
            ui,
            "CCT (Temp)",
            metrics.cct_tint.0,
            2000.0,
            12000.0,
            "K",
            egui::Color32::from_rgb(255, 200, 150),
        );

        // Tint needs centered gauge
        ui.horizontal(|ui| {
            ui.label("Tint");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{:.4}", metrics.cct_tint.1));
            });
        });
        let tint_val = metrics.cct_tint.1;
        let tint_range = 0.1; // +/- 0.1
        let tint_norm = (tint_val / tint_range).clamp(-1.0, 1.0); // -1 to 1
        let desired_size = egui::vec2(ui.available_width(), 6.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_gray(40));
        let center_x = rect.center().x;
        let bar_width = (tint_norm * rect.width() / 2.0).abs();
        let bar_rect = if tint_norm < 0.0 {
            egui::Rect::from_min_size(
                egui::Pos2::new(center_x - bar_width, rect.min.y),
                egui::vec2(bar_width, rect.height()),
            )
        } else {
            egui::Rect::from_min_size(
                egui::Pos2::new(center_x, rect.min.y),
                egui::vec2(bar_width, rect.height()),
            )
        };
        let tint_color = if tint_norm < 0.0 {
            egui::Color32::GREEN
        } else {
            egui::Color32::from_rgb(255, 0, 255)
        };
        ui.painter().rect_filled(bar_rect, 3.0, tint_color);
        ui.add_space(4.0);

        gauge(
            ui,
            "Saturation",
            metrics.saturation_mean,
            0.0,
            100.0,
            "",
            egui::Color32::from_rgb(200, 50, 200),
        );

        ui.label("Lab Color Space (a* vs b*):");
        Plot::new("lab_plot")
            .view_aspect(1.0)
            .data_aspect(1.0)
            .include_x(-60.0)
            .include_x(60.0)
            .include_y(-60.0)
            .include_y(60.0)
            .show(ui, |plot_ui| {
                let points = PlotPoints::from(vec![[
                    metrics.lab_mean[1] as f64,
                    metrics.lab_mean[2] as f64,
                ]]);
                plot_ui.points(
                    Points::new("mean_lab", points)
                        .radius(6.0)
                        .shape(egui_plot::MarkerShape::Circle)
                        .color(egui::Color32::WHITE)
                        .name("Mean Color"),
                );

                // Draw axes
                plot_ui.line(
                    Line::new(
                        "axis_x",
                        PlotPoints::from(vec![[-128.0, 0.0], [128.0, 0.0]]),
                    )
                    .color(egui::Color32::DARK_GRAY),
                );
                plot_ui.line(
                    Line::new(
                        "axis_y",
                        PlotPoints::from(vec![[0.0, -128.0], [0.0, 128.0]]),
                    )
                    .color(egui::Color32::DARK_GRAY),
                );
            });
    });

    ui.separator();

    ui.collapsing("Texture & Grain", |ui| {
        gauge(
            ui,
            "Laplacian Var",
            metrics.laplacian_variance,
            0.0,
            1000.0,
            "",
            egui::Color32::LIGHT_GRAY,
        );
        gauge(
            ui,
            "PSD Slope (Beta)",
            metrics.psd_slope,
            0.0,
            4.0,
            "",
            egui::Color32::YELLOW,
        );

        ui.label("LBP Histogram (Texture Pattern):");
        Plot::new("lbp_hist")
            .view_aspect(2.0)
            .include_y(0.0)
            .include_y(1.0)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                let bars: Vec<Bar> = metrics
                    .lbp_hist
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| {
                        Bar::new(i as f64, v as f64)
                            .fill(egui::Color32::LIGHT_BLUE)
                            .width(0.8)
                    })
                    .collect();
                plot_ui.bar_chart(BarChart::new("lbp_bars", bars));
            });

        ui.label("GLCM (Co-occurrence):");
        ui.horizontal(|ui| {
            ui.label(format!("Contrast: {:.2}", metrics.glcm_stats[0]));
        });
        gauge(
            ui,
            "Correlation",
            metrics.glcm_stats[1],
            -1.0,
            1.0,
            "",
            egui::Color32::LIGHT_BLUE,
        );
        gauge(
            ui,
            "Energy",
            metrics.glcm_stats[2],
            0.0,
            1.0,
            "",
            egui::Color32::GOLD,
        );
        gauge(
            ui,
            "Homogeneity",
            metrics.glcm_stats[3],
            0.0,
            1.0,
            "",
            egui::Color32::GREEN,
        );
    });

    ui.separator();

    // 4. Statistics
    ui.collapsing("RGB Statistics", |ui| {
        Plot::new("rgb_stats")
            .view_aspect(1.5)
            .legend(Legend::default())
            .include_y(0.0)
            .include_y(255.0)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                let means = metrics.mean_rgb;
                let stds = metrics.std_rgb;

                let bars = vec![
                    Bar::new(0.0, means[0] as f64)
                        .name("Red")
                        .fill(egui::Color32::RED)
                        .width(0.4),
                    Bar::new(1.0, means[1] as f64)
                        .name("Green")
                        .fill(egui::Color32::GREEN)
                        .width(0.4),
                    Bar::new(2.0, means[2] as f64)
                        .name("Blue")
                        .fill(egui::Color32::BLUE)
                        .width(0.4),
                ];
                plot_ui.bar_chart(BarChart::new("rgb_means", bars));

                // Error bars
                for i in 0..3 {
                    let x = i as f64;
                    let y = means[i] as f64;
                    let s = stds[i] as f64;
                    plot_ui.line(
                        Line::new(
                            format!("err_v_{}", i),
                            PlotPoints::from(vec![[x, y - s], [x, y + s]]),
                        )
                        .color(egui::Color32::WHITE)
                        .name(""),
                    );
                    plot_ui.line(
                        Line::new(
                            format!("err_t_{}", i),
                            PlotPoints::from(vec![[x - 0.1, y - s], [x + 0.1, y - s]]),
                        )
                        .color(egui::Color32::WHITE)
                        .name(""),
                    );
                    plot_ui.line(
                        Line::new(
                            format!("err_b_{}", i),
                            PlotPoints::from(vec![[x - 0.1, y + s], [x + 0.1, y + s]]),
                        )
                        .color(egui::Color32::WHITE)
                        .name(""),
                    );
                }
            });

        ui.label("Skewness & Kurtosis:");
        Plot::new("skew_kurt")
            .view_aspect(2.0)
            .legend(Legend::default())
            .include_y(-5.0)
            .include_y(5.0)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                let mut bars_skew = Vec::new();
                let mut bars_kurt = Vec::new();

                for i in 0..3 {
                    let color = match i {
                        0 => egui::Color32::RED,
                        1 => egui::Color32::GREEN,
                        _ => egui::Color32::BLUE,
                    };
                    bars_skew.push(
                        Bar::new(i as f64, metrics.skewness_rgb[i] as f64)
                            .fill(color)
                            .width(0.3)
                            .name(match i {
                                0 => "R Skew",
                                1 => "G Skew",
                                _ => "B Skew",
                            }),
                    );
                    bars_kurt.push(
                        Bar::new((i + 4) as f64, metrics.kurtosis_rgb[i] as f64)
                            .fill(color)
                            .width(0.3)
                            .name(match i {
                                0 => "R Kurt",
                                1 => "G Kurt",
                                _ => "B Kurt",
                            }),
                    );
                }
                plot_ui.bar_chart(BarChart::new("skew_bars", bars_skew));
                plot_ui.bar_chart(BarChart::new("kurt_bars", bars_kurt));
            });

        ui.separator();

        ui.collapsing("RGB Quantiles", |ui| {
            egui::Grid::new("quantiles_grid")
                .striped(true)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Channel").strong());
                    ui.label(egui::RichText::new("P10").strong());
                    ui.label(egui::RichText::new("P50").strong());
                    ui.label(egui::RichText::new("P90").strong());
                    ui.label(egui::RichText::new("P99").strong());
                    ui.end_row();

                    let names = ["Red", "Green", "Blue"];
                    for (i, name) in names.iter().enumerate() {
                        ui.label(*name);
                        for q in 0..4 {
                            ui.label(format!("{}", metrics.quantiles_rgb[i][q]));
                        }
                        ui.end_row();
                    }
                });
        });
    });
}
