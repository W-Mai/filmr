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
