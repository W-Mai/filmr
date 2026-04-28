mod preset_io;
mod professional;
mod shutter_speed;
mod simple;

use egui::{Context, RichText};

use crate::config::UxMode;
use crate::ui::app::{FilmrApp, RightTab};

pub use shutter_speed::ShutterSpeed;

/// Section header — uppercase, small, muted color (matches mockup).
pub(super) fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .strong()
            .size(12.0)
            .color(egui::Color32::from_rgb(90, 90, 100)),
    );
    ui.add_space(3.0);
}

/// Render left panel (film list + style) and right panel (adjustment tabs).
pub fn render_controls(app: &mut FilmrApp, ctx: &Context) {
    let mut changed = false;

    // ── Left Panel: Film List + Style ──
    egui::SidePanel::left("film_list_panel")
        .default_width(224.0)
        .min_width(200.0)
        .max_width(300.0)
        .show(ctx, |ui| {
            // Style fixed at bottom (mockup: border-top separated)
            egui::TopBottomPanel::bottom("style_panel").show_inside(ui, |ui| {
                ui.separator();
                ui.add_space(4.0);
                simple::render_style_selector(app, ui, &mut changed);
                ui.add_space(4.0);
            });

            // Film list scrollable above
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);
                simple::render_film_list(app, ui, &mut changed);
                ui.add_space(8.0);
            });
        });

    // ── Right Panel: Adjustments ──
    egui::SidePanel::right("adjust_panel")
        .default_width(280.0)
        .min_width(260.0)
        .max_width(360.0)
        .show(ctx, |ui| {
            // Mode switch — mockup: selected=accent bg + dark text, unselected=medium bg + secondary text
            let accent = egui::Color32::from_rgb(230, 155, 50);
            let bg_medium = egui::Color32::from_rgb(42, 42, 48);
            let text_dark = egui::Color32::from_rgb(24, 24, 28);
            let text_secondary = egui::Color32::from_rgb(150, 150, 160);

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    let prev_mode = app.ux_mode;
                    let is_simple = app.ux_mode == UxMode::Simple;
                    let simple_btn = egui::Button::new(
                        egui::RichText::new("Simple")
                            .size(12.0)
                            .strong()
                            .color(if is_simple { text_dark } else { text_secondary }),
                    )
                    .fill(if is_simple { accent } else { bg_medium })
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(4.0);
                    if ui.add(simple_btn).clicked() {
                        app.ux_mode = UxMode::Simple;
                        app.right_tab = RightTab::Adjust;
                    }

                    let pro_btn = egui::Button::new(
                        egui::RichText::new("Professional")
                            .size(12.0)
                            .strong()
                            .color(if !is_simple {
                                text_dark
                            } else {
                                text_secondary
                            }),
                    )
                    .fill(if !is_simple { accent } else { bg_medium })
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(4.0);
                    if ui.add(pro_btn).clicked() {
                        app.ux_mode = UxMode::Professional;
                    }

                    if prev_mode != app.ux_mode {
                        if let Some(cm) = &mut app.config_manager {
                            cm.config.ux_mode = app.ux_mode;
                            cm.save();
                        }
                        changed = true;
                    }
                });
            });
            ui.separator();

            // Tab bar (Professional only) — each tab fills equal width
            if app.ux_mode == UxMode::Professional {
                let accent = egui::Color32::from_rgb(230, 155, 50);
                let text_secondary = egui::Color32::from_rgb(150, 150, 160);
                let tab_width = (ui.available_width() / 3.0).min(100.0);
                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (RightTab::Adjust, "Adjust"),
                        (RightTab::Effects, "Effects"),
                        (RightTab::Detail, "Detail"),
                    ] {
                        let is_active = app.right_tab == tab;
                        let text = egui::RichText::new(label).size(12.0).color(if is_active {
                            accent
                        } else {
                            text_secondary
                        });
                        let text = if is_active { text.strong() } else { text };
                        let btn = egui::Button::new(text)
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(tab_width, 0.0));
                        let response = ui.add(btn);
                        if is_active {
                            let rect = response.rect;
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(rect.left(), rect.bottom() - 2.0),
                                    egui::vec2(rect.width(), 2.0),
                                ),
                                0.0,
                                accent,
                            );
                        }
                        if response.clicked() {
                            app.right_tab = tab;
                        }
                    }
                });
                ui.separator();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(8, 0))
                    .show(ui, |ui| match app.right_tab {
                        RightTab::Adjust => {
                            render_adjust_tab(app, ui, ctx, &mut changed);
                        }
                        RightTab::Effects => {
                            professional::render_effects_tab(app, ui, &mut changed);
                        }
                        RightTab::Detail => {
                            professional::render_detail_tab(app, ui, &mut changed);
                        }
                    });
            });
        });

    if changed {
        app.process_and_update_texture(ctx);
        app.regenerate_thumbnails();
    }
}

/// Adjust tab — shown in both Simple and Professional modes.
fn render_adjust_tab(app: &mut FilmrApp, ui: &mut egui::Ui, _ctx: &Context, changed: &mut bool) {
    // Exposure
    section_header(ui, "EXPOSURE");
    if app.ux_mode == UxMode::Professional {
        use super::controls::shutter_speed::ShutterSpeed;
        ui.horizontal(|ui| {
            ui.label("Exposure Time");
            let mut shutter = ShutterSpeed(app.exposure_time as f64);
            if shutter.ui(ui).changed() {
                *changed = true;
            }
            app.exposure_time = shutter.0 as f32;
        });
    } else if ui
        .add(
            egui::Slider::new(&mut app.exposure_time, 0.001..=30.0)
                .logarithmic(true)
                .text("☀ Brightness"),
        )
        .changed()
    {
        *changed = true;
    }
    if ui
        .add(egui::Slider::new(&mut app.gamma_boost, 0.5..=2.0).text("◑ Contrast"))
        .changed()
    {
        *changed = true;
    }
    ui.separator();

    // Color
    section_header(ui, "COLOR");
    if ui
        .add(egui::Slider::new(&mut app.warmth, -1.0..=1.0).text("🔥 Warmth"))
        .changed()
    {
        *changed = true;
    }
    if ui
        .add(egui::Slider::new(&mut app.saturation, 0.0..=2.0).text("🌈 Intensity"))
        .changed()
    {
        *changed = true;
    }
    ui.separator();

    // Auto — same row
    ui.horizontal(|ui| {
        if ui.checkbox(&mut app.auto_levels, "🎚 Auto Levels").changed() {
            *changed = true;
        }
        if ui.button("✨ Auto Enhance").clicked() {
            app.white_balance_mode = filmr::WhiteBalanceMode::Auto;
            app.white_balance_strength = 1.0;
            *changed = true;
        }
    });
    ui.separator();

    // Professional-only: WB + Output
    if app.ux_mode == UxMode::Professional {
        professional::render_white_balance(app, ui, changed);
        ui.separator();
        professional::render_output_mode(app, ui, changed);
    }
}
