use egui::{Context, Vec2, Pos2, Rect, Sense};
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints, Points};
use filmr::{OutputMode, WhiteBalanceMode};
use crate::app::FilmrApp;

pub fn render_controls(app: &mut FilmrApp, ctx: &Context) {
    egui::SidePanel::left("controls_panel").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Filmr Controls");
            ui.separator();

            let mut changed = false;

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
                    let mut groups: std::collections::BTreeMap<String, Vec<usize>> = Default::default();
                    for (idx, (name, _)) in app.stocks.iter().enumerate() {
                        let brand = name.split_whitespace().next().unwrap_or("Other").to_string();
                        groups.entry(brand).or_default().push(idx);
                    }

                    for (brand, indices) in groups {
                        ui.collapsing(brand, |ui| {
                            for idx in indices {
                                let (name, _) = app.stocks[idx];
                                if ui.selectable_value(&mut app.selected_stock_idx, idx, name).clicked() {
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
                .add(
                    egui::Slider::new(&mut app.halation_threshold, 0.0..=1.0)
                        .text("Threshold"),
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

            ui.group(|ui| {
                let stock = app.get_current_stock();
                ui.label("Grain (From Preset)");
                ui.label(format!("Alpha: {:.3}", stock.grain_model.alpha));
                ui.label(format!("Sigma: {:.3}", stock.grain_model.sigma_read));
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
                    // If we have a developed image, we should probably show it?
                    // But currently we show preview metrics if params changed.
                    // Let's stick to preview metrics unless we want to be fancy.
                    &app.metrics_preview
                } else {
                    &app.metrics_preview
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(metrics) = metrics_to_show {
                        
                        // Helper for simple gauges
                        let gauge = |ui: &mut egui::Ui, name: &str, val: f32, min: f32, max: f32, unit: &str, color: egui::Color32| {
                            ui.horizontal(|ui| {
                                ui.label(name);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(format!("{:.2} {}", val, unit));
                                });
                            });
                            let progress = ((val - min) / (max - min)).clamp(0.0, 1.0);
                            let desired_size = egui::vec2(ui.available_width(), 6.0);
                            let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
                            ui.painter().rect_filled(rect, 3.0, egui::Color32::from_gray(40));
                            let fill_width = rect.width() * progress;
                            let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
                            ui.painter().rect_filled(fill_rect, 3.0, color);
                            ui.add_space(4.0);
                        };

                        // 1. Histogram & Exposure
                        ui.collapsing("Histogram & Exposure", |ui| {
                            gauge(ui, "Dynamic Range", metrics.dynamic_range, 0.0, 15.0, "dB", egui::Color32::GOLD);
                            gauge(ui, "Entropy", metrics.entropy, 0.0, 8.0, "bits", egui::Color32::LIGHT_BLUE);
                            
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
                                        Bar::new(0.0, zeros as f64).name("Blacks").fill(egui::Color32::RED).width(0.6),
                                        Bar::new(1.0, saturated as f64).name("Whites").fill(egui::Color32::WHITE).width(0.6),
                                    ];
                                    plot_ui.bar_chart(BarChart::new("clipping_bars", bars));
                                });

                            ui.label("RGB Histogram:");
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut app.hist_log_scale, "Log Scale");
                                ui.checkbox(&mut app.hist_clamp_zeros, "Ignore Blacks (0)");
                            });

                            Plot::new("rgb_hist")
                                .view_aspect(1.5)
                                .legend(Legend::default())
                                .include_y(0.0)
                                .include_y(1.05) // Leave some headroom
                                .allow_zoom(false)
                                .allow_drag(false)
                                .allow_scroll(false)
                                .show(ui, |plot_ui| {
                                    // 1. Collect all relevant bin counts to find a robust maximum (99.5th percentile)
                                    // This avoids single-bin spikes (like pure black/white) compressing the whole chart.
                                    let mut all_counts = Vec::with_capacity(256 * 3);
                                    for c in 0..3 {
                                        for (i, &v) in metrics.hist_rgb[c].iter().enumerate() {
                                            if !app.hist_clamp_zeros || i > 0 {
                                                all_counts.push(v as f64);
                                            }
                                        }
                                    }
                                    // Sort to find percentile
                                    all_counts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                                    
                                    // Get 99.5th percentile value
                                    let max_val = if all_counts.is_empty() {
                                        1.0
                                    } else {
                                        let idx = ((all_counts.len() as f64) * 0.995) as usize;
                                        all_counts[idx.min(all_counts.len() - 1)].max(1.0)
                                    };
                                    
                                    let norm_denom = if app.hist_log_scale {
                                        (max_val + 1.0).log10()
                                    } else {
                                        max_val
                                    };

                                    for (c, color) in [(0, egui::Color32::RED), (1, egui::Color32::GREEN), (2, egui::Color32::BLUE)].iter() {
                                        // Construct line points explicitly
                                        let mut line_points: Vec<[f64; 2]> = Vec::with_capacity(256);
                                        
                                        for (i, &v) in metrics.hist_rgb[*c].iter().enumerate() {
                                            if app.hist_clamp_zeros && i == 0 {
                                                continue;
                                            }
                                            
                                            let val_raw = if app.hist_log_scale {
                                                (v as f64 + 1.0).log10()
                                            } else {
                                                v as f64
                                            };
                                            
                                            // Clamp to slightly above 1.0 so we see flat tops for clipped spikes
                                            let val_norm = (val_raw / norm_denom).min(1.0);
                                            line_points.push([i as f64, val_norm]);
                                        }

                                        // Draw Line on top
                                        if !line_points.is_empty() {
                                            plot_ui.line(Line::new(format!("hist_{}", c), PlotPoints::new(line_points))
                                                .color(*color)
                                                .name(match c { 0 => "Red", 1 => "Green", _ => "Blue" }));
                                        }
                                    }
                                });
                        });

                        ui.separator();

                        // 2. Color Analysis
                        ui.collapsing("Color Analysis", |ui| {
                            gauge(ui, "CCT (Temp)", metrics.cct_tint.0, 2000.0, 12000.0, "K", egui::Color32::from_rgb(255, 200, 150));
                            
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
                            ui.painter().rect_filled(rect, 3.0, egui::Color32::from_gray(40));
                            let center_x = rect.center().x;
                            let bar_width = (tint_norm * rect.width() / 2.0).abs();
                            let bar_rect = if tint_norm < 0.0 {
                                egui::Rect::from_min_size(egui::Pos2::new(center_x - bar_width, rect.min.y), egui::vec2(bar_width, rect.height()))
                            } else {
                                egui::Rect::from_min_size(egui::Pos2::new(center_x, rect.min.y), egui::vec2(bar_width, rect.height()))
                            };
                            let tint_color = if tint_norm < 0.0 { egui::Color32::GREEN } else { egui::Color32::from_rgb(255, 0, 255) };
                            ui.painter().rect_filled(bar_rect, 3.0, tint_color);
                            ui.add_space(4.0);

                            gauge(ui, "Saturation", metrics.saturation_mean, 0.0, 100.0, "", egui::Color32::from_rgb(200, 50, 200));
                            ui.horizontal(|ui| {
                                ui.label(format!("Sat Skew: {:.2}", metrics.saturation_skew));
                                ui.add_space(10.0);
                                ui.label(format!("R/G: {:.2}", metrics.rg_ratio));
                                ui.label(format!("B/G: {:.2}", metrics.bg_ratio));
                            });
                            
                            ui.label("Lab Color Space (a* vs b*):");
                            Plot::new("lab_plot")
                                .view_aspect(1.0)
                                .data_aspect(1.0)
                                .include_x(-60.0).include_x(60.0)
                                .include_y(-60.0).include_y(60.0)
                                .show(ui, |plot_ui| {
                                    let points = PlotPoints::from(vec![[metrics.lab_mean[1] as f64, metrics.lab_mean[2] as f64]]);
                                    plot_ui.points(Points::new("mean_lab", points).radius(6.0).shape(egui_plot::MarkerShape::Circle).color(egui::Color32::WHITE).name("Mean Color"));
                                    
                                    // Draw axes
                                    plot_ui.line(Line::new("axis_x", PlotPoints::from(vec![[-128.0, 0.0], [128.0, 0.0]])).color(egui::Color32::DARK_GRAY));
                                    plot_ui.line(Line::new("axis_y", PlotPoints::from(vec![[0.0, -128.0], [0.0, 128.0]])).color(egui::Color32::DARK_GRAY));
                                });
                        });

                        ui.separator();

                        // 3. Texture & Structure
                        ui.collapsing("Texture & Grain", |ui| {
                            gauge(ui, "Laplacian Var", metrics.laplacian_variance, 0.0, 1000.0, "", egui::Color32::LIGHT_GRAY);
                            gauge(ui, "PSD Slope (Beta)", metrics.psd_slope, 0.0, 4.0, "", egui::Color32::YELLOW);
                            
                            ui.label("LBP Histogram (Texture Pattern):");
                            Plot::new("lbp_hist")
                                .view_aspect(2.0)
                                .include_y(0.0)
                                .include_y(1.0)
                                .allow_zoom(false)
                                .allow_drag(false)
                                .allow_scroll(false)
                                .show(ui, |plot_ui| {
                                    let bars: Vec<Bar> = metrics.lbp_hist.iter().enumerate().map(|(i, &v)| {
                                        Bar::new(i as f64, v as f64).fill(egui::Color32::LIGHT_BLUE).width(0.8)
                                    }).collect();
                                    plot_ui.bar_chart(BarChart::new("lbp_bars", bars));
                                });
                                
                            ui.label("GLCM (Co-occurrence):");
                            ui.horizontal(|ui| {
                                ui.label(format!("Contrast: {:.2}", metrics.glcm_stats[0]));
                            });
                            gauge(ui, "Correlation", metrics.glcm_stats[1], -1.0, 1.0, "", egui::Color32::LIGHT_BLUE);
                            gauge(ui, "Energy", metrics.glcm_stats[2], 0.0, 1.0, "", egui::Color32::GOLD);
                            gauge(ui, "Homogeneity", metrics.glcm_stats[3], 0.0, 1.0, "", egui::Color32::GREEN);
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
                                        Bar::new(0.0, means[0] as f64).name("Red").fill(egui::Color32::RED).width(0.4),
                                        Bar::new(1.0, means[1] as f64).name("Green").fill(egui::Color32::GREEN).width(0.4),
                                        Bar::new(2.0, means[2] as f64).name("Blue").fill(egui::Color32::BLUE).width(0.4),
                                    ];
                                    plot_ui.bar_chart(BarChart::new("rgb_means", bars));
                                    
                                    // Error bars
                                    for i in 0..3 {
                                        let x = i as f64;
                                        let y = means[i] as f64;
                                        let s = stds[i] as f64;
                                        plot_ui.line(Line::new(format!("err_v_{}", i), PlotPoints::from(vec![[x, y-s], [x, y+s]])).color(egui::Color32::WHITE).name(""));
                                        plot_ui.line(Line::new(format!("err_t_{}", i), PlotPoints::from(vec![[x-0.1, y-s], [x+0.1, y-s]])).color(egui::Color32::WHITE).name(""));
                                        plot_ui.line(Line::new(format!("err_b_{}", i), PlotPoints::from(vec![[x-0.1, y+s], [x+0.1, y+s]])).color(egui::Color32::WHITE).name(""));
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
                                            let color = match i { 0 => egui::Color32::RED, 1 => egui::Color32::GREEN, _ => egui::Color32::BLUE };
                                            bars_skew.push(Bar::new(i as f64, metrics.skewness_rgb[i] as f64).fill(color).width(0.3).name(match i {0=>"R Skew", 1=>"G Skew", _=>"B Skew"}));
                                            bars_kurt.push(Bar::new((i+4) as f64, metrics.kurtosis_rgb[i] as f64).fill(color).width(0.3).name(match i {0=>"R Kurt", 1=>"G Kurt", _=>"B Kurt"}));
                                    }
                                    plot_ui.bar_chart(BarChart::new("skew_bars", bars_skew));
                                    plot_ui.bar_chart(BarChart::new("kurt_bars", bars_kurt));
                                });
                                
                            ui.collapsing("RGB Quantiles", |ui| {
                                egui::Grid::new("quantiles_grid").striped(true).spacing([20.0, 4.0]).show(ui, |ui| {
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

                    } else {
                        ui.label("No metrics available. Load an image.");
                    }
                });
            });
    }
}

pub fn render_central_panel(app: &mut FilmrApp, ctx: &Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Toolbar Overlay
        ui.horizontal(|ui| {
            // Hold to Compare (Larger and Conspicuous)
            app.show_original = ui.add_sized(
                [150.0, 40.0],
                egui::Button::new("HOLD TO COMPARE").min_size(Vec2::new(150.0, 40.0)),
            ).is_pointer_button_down_on();
            
            ui.separator();

            // Toggle Metrics Panel
            ui.toggle_value(&mut app.show_metrics, "Metrics Panel");

            ui.separator();

            if ui.add_sized([100.0, 40.0], egui::Button::new("Develop")).clicked() {
                app.develop_image(ctx);
            }

            let save_btn = egui::Button::new("Save").min_size(Vec2::new(100.0, 40.0));
            if ui.add_enabled(app.developed_image.is_some(), save_btn).clicked() {
                app.save_image();
            }
        });
        ui.separator();

        let texture_to_show = if app.show_original {
            app.original_texture.as_ref()
        } else {
            app.processed_texture.as_ref()
        };

        if let Some(texture) = texture_to_show {
            // Interactive Area
            let rect = ui.available_rect_before_wrap();
            let response =
                ui.interact(rect, ui.id().with("image_area"), Sense::click_and_drag());

            // 1. Handle Zoom (Pinch or Ctrl+Scroll)
            let zoom_delta = ctx.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                // Zoom towards mouse pointer
                if let Some(pointer_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    let center = rect.center();
                    let pointer_in_layer = pointer_pos - center;
                    let offset_to_pointer = pointer_in_layer - app.offset;

                    app.offset -= offset_to_pointer * (zoom_delta - 1.0);
                    app.zoom *= zoom_delta;
                } else {
                    app.zoom *= zoom_delta;
                }
            }

            // 2. Handle Pan (Drag)
            if response.dragged() {
                app.offset += response.drag_delta();
            }

            // 3. Double Click to Reset
            if response.double_clicked() {
                app.zoom = 1.0;
                app.offset = Vec2::ZERO;
            }

            // 4. Draw Image
            let image_size = texture.size_vec2();
            let aspect = image_size.x / image_size.y;
            let view_aspect = rect.width() / rect.height();

            let base_scale = if aspect > view_aspect {
                rect.width() / image_size.x
            } else {
                rect.height() / image_size.y
            };

            let current_scale = base_scale * app.zoom;
            let displayed_size = image_size * current_scale;

            let center = rect.center() + app.offset;
            let image_rect = Rect::from_center_size(center, displayed_size);

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
