use filmr::FilmStyle;

use crate::ui::app::FilmrApp;

use crate::ui::components::{
    collapsing_section, pill_selector_rows, section_header, ACCENT, TEXT_DISABLED, TEXT_PRIMARY,
    TEXT_SECONDARY,
};

/// Render the film stock list (grouped by brand with thumbnails).
pub fn render_film_list(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    section_header(ui, "🎞 FILM STOCK");
    ui.separator();

    let mut preset_changed = false;
    let mut enter_studio_idx: Option<usize> = None;
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.set_min_size(ui.available_size());

                let mut groups: std::collections::BTreeMap<String, Vec<usize>> = Default::default();
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
                    collapsing_section(ui, &brand.to_uppercase(), true, |ui| {
                        for idx in indices {
                            let stock = &app.stocks[idx];
                            let full_name = &stock.full_name();
                            let name = &stock.name;
                            let is_selected = app.selected_stock_idx == idx;

                            let padding = 6.0f32;
                            let thumb_w = 32.0f32;
                            let thumb_h = 24.0f32;
                            let row_height = thumb_h + padding * 2.0;
                            let corner_radius = 4.0f32;
                            let inner_radius = 3.0f32;

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
                                let img_aspect = thumb.size()[0] as f32 / thumb.size()[1] as f32;
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
                                ACCENT
                            } else if response.hovered() {
                                TEXT_PRIMARY
                            } else {
                                TEXT_SECONDARY
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

                            // Double-click → enter Stock Studio
                            if response.double_clicked() {
                                app.selected_stock_idx = idx;
                                preset_changed = true;
                                enter_studio_idx = Some(idx);
                            }

                            ui.add_space(2.0);
                        }
                    });
                }
            });
        });

    if preset_changed {
        app.load_preset_values();
        *changed = true;
    }

    // Enter Stock Studio on double-click
    if let Some(idx) = enter_studio_idx {
        use crate::config::AppMode;
        if idx < app.builtin_stock_count {
            // Fork builtin → create custom copy → edit
            let mut forked = app.stocks[idx].as_ref().clone();
            forked.name = format!("{} (Custom)", forked.name);
            app.stocks.push(std::rc::Rc::from(forked.clone()));
            let new_idx = app.stocks.len() - 1;
            app.selected_stock_idx = new_idx;
            app.studio_stock = app.stocks[new_idx].as_ref().clone();
            app.studio_stock_idx = Some(new_idx);
        } else {
            // Custom stock → edit directly
            app.studio_stock = app.stocks[idx].as_ref().clone();
            app.studio_stock_idx = Some(idx);
        }
        app.mode = AppMode::StockStudio;
        app.has_unsaved_changes = true;
    }
}

/// Render the rendering style selector — two-row pill/segmented control.
pub fn render_style_selector(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    section_header(ui, "🎨 STYLE");

    let styles = FilmStyle::all();
    let row1: Vec<(FilmStyle, &str)> = styles[..3].iter().map(|s| (*s, s.name())).collect();
    let row2: Vec<(FilmStyle, &str)> = styles[3..]
        .iter()
        .map(|s| {
            let name = match s {
                FilmStyle::HighContrast => "Hi-Con",
                other => other.name(),
            };
            (*s, name)
        })
        .collect();
    let rows: &[&[(FilmStyle, &str)]] = &[&row1, &row2];

    if pill_selector_rows(ui, "style", &mut app.film_style, rows) {
        *changed = true;
    }

    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(app.film_style.short_description())
            .size(11.0)
            .color(TEXT_DISABLED),
    );
}

/// Render 2D film map — X=saturation, Y=contrast, each film as thumbnail+name.
pub fn render_film_map(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    let rect = ui.available_rect_before_wrap();
    let size = egui::vec2(rect.width(), rect.height().max(300.0));
    let (map_rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    let painter = ui.painter_at(map_rect);

    // Background
    painter.rect_filled(map_rect, 4.0, egui::Color32::from_rgb(28, 28, 32));

    // Grid lines
    let grid_color = egui::Color32::from_rgb(40, 40, 46);
    for i in 1..4 {
        let t = i as f32 / 4.0;
        let x = map_rect.left() + t * map_rect.width();
        let y = map_rect.top() + t * map_rect.height();
        painter.line_segment(
            [
                egui::pos2(x, map_rect.top()),
                egui::pos2(x, map_rect.bottom()),
            ],
            egui::Stroke::new(0.5, grid_color),
        );
        painter.line_segment(
            [
                egui::pos2(map_rect.left(), y),
                egui::pos2(map_rect.right(), y),
            ],
            egui::Stroke::new(0.5, grid_color),
        );
    }

    // Axis labels
    let label_color = egui::Color32::from_rgb(90, 90, 100);
    painter.text(
        egui::pos2(map_rect.center().x, map_rect.bottom() - 4.0),
        egui::Align2::CENTER_BOTTOM,
        "Saturation \u{2192}",
        egui::FontId::proportional(10.0),
        label_color,
    );
    painter.text(
        egui::pos2(map_rect.left() + 4.0, map_rect.center().y),
        egui::Align2::LEFT_CENTER,
        "Contrast \u{2191}",
        egui::FontId::proportional(10.0),
        label_color,
    );

    // Plot each film stock
    let accent = ACCENT;
    let text_color = egui::Color32::from_rgb(150, 150, 160);
    let margin = 20.0;
    let inner = map_rect.shrink(margin);

    for (idx, stock) in app.stocks.iter().enumerate() {
        let sat = stock.saturation_score();
        let con = stock.contrast_score();
        let x = inner.left() + sat * inner.width();
        let y = inner.bottom() - con * inner.height();
        let is_selected = app.selected_stock_idx == idx;

        // Brand color for placeholder bg and name label
        let brand_color = match stock.manufacturer.as_str() {
            "Kodak" => egui::Color32::from_rgb(255, 210, 0), // Kodak yellow
            "Fujifilm" => egui::Color32::from_rgb(0, 160, 80), // Fujifilm green
            "Ilford" => egui::Color32::from_rgb(180, 180, 180), // Ilford silver
            "Agfa" => egui::Color32::from_rgb(220, 60, 60),  // Agfa red
            "Polaroid" => egui::Color32::from_rgb(100, 160, 220), // Polaroid blue
            _ => text_color,
        };

        // Thumbnail size
        let thumb_w = 40.0f32;
        let thumb_h = 30.0f32;
        let item_rect =
            egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(thumb_w, thumb_h));

        // Draw thumbnail or placeholder
        let full_name = stock.full_name();
        if let Some(thumb) = app.preset_thumbnails.get(&full_name) {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(thumb.id(), item_rect, uv, egui::Color32::WHITE);
        } else {
            // Brand-colored placeholder with subtle bg
            let bg = egui::Color32::from_rgba_unmultiplied(
                brand_color.r() / 4,
                brand_color.g() / 4,
                brand_color.b() / 4,
                255,
            );
            painter.rect_filled(item_rect, 3.0, bg);
            // Brand color top bar
            let bar = egui::Rect::from_min_size(item_rect.min, egui::vec2(item_rect.width(), 3.0));
            painter.rect_filled(bar, egui::CornerRadius::ZERO, brand_color);
        }

        // Selected border
        if is_selected {
            painter.rect(
                item_rect,
                3.0,
                egui::Color32::TRANSPARENT,
                egui::Stroke::new(2.0, accent),
                egui::StrokeKind::Outside,
            );
        }

        // Name below thumbnail — brand color when not selected
        painter.text(
            egui::pos2(x, item_rect.bottom() + 2.0),
            egui::Align2::CENTER_TOP,
            &stock.name,
            egui::FontId::proportional(9.0),
            if is_selected { accent } else { brand_color },
        );
    }

    // Handle click — find nearest stock
    if response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            let mut best_idx = app.selected_stock_idx;
            let mut best_dist = f32::MAX;
            for (idx, stock) in app.stocks.iter().enumerate() {
                let sat = stock.saturation_score();
                let con = stock.contrast_score();
                let x = inner.left() + sat * inner.width();
                let y = inner.bottom() - con * inner.height();
                let center = egui::pos2(x, y);
                let item_rect = egui::Rect::from_center_size(center, egui::vec2(40.0, 30.0));
                if item_rect.expand(5.0).contains(pointer) {
                    let dist = (pointer - center).length();
                    if dist < best_dist {
                        best_dist = dist;
                        best_idx = idx;
                    }
                }
            }
            if best_dist < f32::MAX && best_idx != app.selected_stock_idx {
                app.selected_stock_idx = best_idx;
                app.load_preset_values();
                *changed = true;
            }
        }
    }
}
