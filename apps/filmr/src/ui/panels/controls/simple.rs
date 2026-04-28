use filmr::FilmStyle;

use crate::ui::app::FilmrApp;

use super::section_header;

/// Render the film stock list (grouped by brand with thumbnails).
pub fn render_film_list(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    section_header(ui, "🎞 FILM STOCK");

    let mut preset_changed = false;
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 120.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_size(ui.available_size());

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
                                egui::RichText::new(brand.to_uppercase())
                                    .strong()
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(90, 90, 100)),
                            )
                            .default_open(true)
                            .show(ui, |ui| {
                                for idx in indices {
                                    let stock = &app.stocks[idx];
                                    let full_name = &stock.full_name();
                                    let name = &stock.name;
                                    let is_selected = app.selected_stock_idx == idx;

                                    let padding = 4.0f32;
                                    let thumb_w = 48.0f32;
                                    let thumb_h = thumb_w / 3.0 * 2.0;
                                    let row_height = thumb_h + padding * 2.0;
                                    let corner_radius = 6.0f32;
                                    let inner_radius = corner_radius - padding;

                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), row_height),
                                        egui::Sense::click(),
                                    );

                                    let thumb_rect = egui::Rect::from_min_size(
                                        rect.min + egui::vec2(padding, padding),
                                        egui::vec2(thumb_w, thumb_h),
                                    );

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

                                    if let Some(thumb) = app.preset_thumbnails.get(full_name) {
                                        let img_aspect =
                                            thumb.size()[0] as f32 / thumb.size()[1] as f32;
                                        let container_aspect = thumb_w / thumb_h;

                                        let (w, h) = if img_aspect > container_aspect {
                                            (thumb_w, thumb_w / img_aspect)
                                        } else {
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

                                    let text_x = rect.min.x + padding + thumb_w + padding * 2.0;
                                    let text_color = if is_selected {
                                        egui::Color32::from_rgb(230, 155, 50) // accent
                                    } else {
                                        egui::Color32::from_rgb(150, 150, 160) // secondary
                                    };
                                    ui.painter().text(
                                        egui::pos2(text_x, rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        name,
                                        egui::FontId::monospace(12.0),
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
}

/// Render the rendering style selector.
pub fn render_style_selector(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    section_header(ui, "🎨 STYLE");

    let accent = egui::Color32::from_rgb(230, 155, 50);
    let bg_medium = egui::Color32::from_rgb(42, 42, 48);
    let text_dark = egui::Color32::from_rgb(24, 24, 28);
    let text_secondary = egui::Color32::from_rgb(150, 150, 160);

    let prev_style = app.film_style;
    ui.horizontal_wrapped(|ui| {
        for style in FilmStyle::all() {
            let is_selected = app.film_style == style;
            let btn =
                egui::Button::new(egui::RichText::new(style.name()).size(10.0).strong().color(
                    if is_selected {
                        text_dark
                    } else {
                        text_secondary
                    },
                ))
                .fill(if is_selected { accent } else { bg_medium })
                .corner_radius(4.0);
            if ui.add(btn).clicked() {
                app.film_style = style;
            }
        }
    });

    if app.film_style != prev_style {
        *changed = true;
    }

    ui.small(app.film_style.short_description());
}
