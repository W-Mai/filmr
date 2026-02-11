use egui::{Context, RichText};
use filmr::WhiteBalanceMode;

use crate::ui::app::FilmrApp;

pub fn render_simple_controls(
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
                                    let is_selected = app.selected_stock_idx == idx;

                                    let padding = 4.0f32;
                                    let thumb_w = 56.0f32;
                                    let thumb_h = thumb_w / 3.0 * 2.0;
                                    let row_height = thumb_h + padding * 2.0;
                                    let corner_radius = 8.0f32;
                                    let inner_radius = corner_radius - padding;

                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), row_height),
                                        egui::Sense::click(),
                                    );

                                    let thumb_rect = egui::Rect::from_min_size(
                                        rect.min + egui::vec2(padding, padding),
                                        egui::vec2(thumb_w, thumb_h),
                                    );

                                    // Draw hover/active/selected background
                                    if response.hovered() || is_selected {
                                        let bg_color = if response.is_pointer_button_down_on() {
                                            ui.visuals().widgets.active.bg_fill
                                        } else if is_selected {
                                            ui.visuals().selection.bg_fill
                                        } else {
                                            ui.visuals().widgets.hovered.bg_fill
                                        };
                                        ui.painter().rect_filled(rect, corner_radius, bg_color);
                                    }

                                    // Draw thumbnail with contain effect (fit within container, preserve aspect ratio)
                                    if let Some(thumb) = app.preset_thumbnails.get(full_name) {
                                        let img_aspect =
                                            thumb.size()[0] as f32 / thumb.size()[1] as f32;
                                        let container_aspect = thumb_w / thumb_h;

                                        let (w, h) = if img_aspect > container_aspect {
                                            // Image is wider than container, fit by width
                                            (thumb_w, thumb_w / img_aspect)
                                        } else {
                                            // Image is taller than container, fit by height
                                            (thumb_h * img_aspect, thumb_h)
                                        };
                                        let img_rect = egui::Rect::from_center_size(
                                            thumb_rect.center(),
                                            egui::vec2(w, h),
                                        );
                                        ui.painter().rect_filled(
                                            thumb_rect,
                                            inner_radius,
                                            egui::Color32::from_gray(60),
                                        );
                                        egui::Image::new(thumb)
                                            .corner_radius(inner_radius)
                                            .paint_at(ui, img_rect);
                                    } else {
                                        ui.painter().rect_filled(
                                            thumb_rect,
                                            inner_radius,
                                            egui::Color32::from_gray(60),
                                        );
                                    }

                                    // Draw label with proper offset
                                    let text_x = rect.min.x + padding + thumb_w + padding * 2.0;
                                    let text_color = if is_selected {
                                        ui.visuals().selection.stroke.color
                                    } else {
                                        ui.visuals().text_color()
                                    };
                                    ui.painter().text(
                                        egui::pos2(text_x, rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        name,
                                        egui::FontId::monospace(14.0),
                                        text_color,
                                    );

                                    if response.clicked() {
                                        app.selected_stock_idx = idx;
                                        preset_changed = true;
                                    }

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
