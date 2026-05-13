mod preset_io;
mod professional;
mod shutter_speed;
mod simple;

use egui::{Context, RichText};
use egui_taffy::{taffy, tui, TuiBuilderLogic};

use crate::config::UxMode;
use crate::ui::app::{FilmrApp, RightTab};
use crate::ui::components::{
    labeled_slider, section_divider, section_header, ACCENT, BG_MEDIUM, TEXT_DARK, TEXT_SECONDARY,
};

pub use shutter_speed::ShutterSpeed;

/// Center arbitrary widgets horizontally using taffy flexbox.
fn centered_horizontal(ui: &mut egui::Ui, id_salt: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    use taffy::prelude::*;
    tui(ui, ui.id().with(id_salt))
        .reserve_available_width()
        .style(taffy::Style {
            display: taffy::Display::Flex,
            flex_direction: taffy::FlexDirection::Row,
            justify_content: Some(taffy::JustifyContent::Center),
            align_items: Some(taffy::AlignItems::Center),
            gap: length(8.0),
            padding: length(6.0),
            size: taffy::Size {
                width: percent(1.0),
                height: auto(),
            },
            ..Default::default()
        })
        .show(|tui| {
            tui.egui_layout(egui::Layout::left_to_right(egui::Align::Center))
                .ui(|ui| {
                    add_contents(ui);
                });
        });
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

    // ── Right Panel: Adjustments (skip in StockStudio mode — studio panel takes over) ──
    if app.mode != crate::config::AppMode::StockStudio {
        egui::SidePanel::right("adjust_panel")
            .default_width(280.0)
            .min_width(260.0)
            .max_width(360.0)
            .show(ctx, |ui| {
                // Mode switch — two centered buttons
                let accent = ACCENT;
                let bg_medium = BG_MEDIUM;
                let text_dark = TEXT_DARK;
                let text_secondary = TEXT_SECONDARY;

                {
                    let prev_mode = app.ux_mode;
                    let is_simple = app.ux_mode == UxMode::Simple;
                    let btn = |selected, label| {
                        egui::Button::new(
                            egui::RichText::new(label)
                                .size(12.0)
                                .strong()
                                .color(if selected { text_dark } else { text_secondary }),
                        )
                        .fill(if selected { accent } else { bg_medium })
                        .stroke(egui::Stroke::NONE)
                        .corner_radius(12.0)
                        .min_size(egui::vec2(0.0, 24.0))
                    };
                    centered_horizontal(ui, "mode_switch", |ui| {
                        if ui.add(btn(is_simple, "Simple")).clicked() {
                            app.ux_mode = UxMode::Simple;
                            app.right_tab = RightTab::Adjust;
                        }
                        if ui.add(btn(!is_simple, "Professional")).clicked() {
                            app.ux_mode = UxMode::Professional;
                        }
                    });
                    if prev_mode != app.ux_mode {
                        if let Some(cm) = &mut app.config_manager {
                            cm.config.ux_mode = app.ux_mode;
                            cm.save();
                        }
                        changed = true;
                    }
                }
                ui.separator();

                // Tab bar (Professional only) — flex-1 equal width tabs
                if app.ux_mode == UxMode::Professional {
                    use taffy::prelude::*;
                    let accent_c = ACCENT;
                    let text_secondary_c = TEXT_SECONDARY;
                    let tabs = [
                        (RightTab::Adjust, "Adjust"),
                        (RightTab::Effects, "Effects"),
                        (RightTab::Detail, "Detail"),
                    ];
                    tui(ui, ui.id().with("tab_bar"))
                        .reserve_available_width()
                        .style(taffy::Style {
                            display: taffy::Display::Flex,
                            flex_direction: taffy::FlexDirection::Row,
                            size: taffy::Size {
                                width: percent(1.0),
                                height: auto(),
                            },
                            ..Default::default()
                        })
                        .show(|tui| {
                            for (tab, label) in tabs {
                                let is_active = app.right_tab == tab;
                                tui.style(taffy::Style {
                                    flex_grow: 1.0,
                                    justify_content: Some(taffy::JustifyContent::Center),
                                    align_items: Some(taffy::AlignItems::Center),
                                    padding: length(8.0),
                                    ..Default::default()
                                })
                                .ui(|ui| {
                                    let text =
                                        egui::RichText::new(label).size(13.0).color(if is_active {
                                            accent_c
                                        } else {
                                            text_secondary_c
                                        });
                                    let text = if is_active { text.strong() } else { text };
                                    let btn = egui::Button::new(text)
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE)
                                        .min_size(egui::vec2(0.0, 24.0));
                                    let response = ui.add(btn);
                                    if is_active {
                                        let rect = response.rect;
                                        ui.painter().rect_filled(
                                            egui::Rect::from_min_size(
                                                egui::pos2(rect.left(), rect.bottom() - 2.0),
                                                egui::vec2(rect.width(), 2.0),
                                            ),
                                            0.0,
                                            accent_c,
                                        );
                                    }
                                    if response.clicked() {
                                        app.right_tab = tab;
                                    }
                                });
                            }
                        });
                    ui.separator();
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(4.0);
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(12, 0))
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
    } // end if not StockStudio

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
        // Label + value row, then full-width slider
        let secondary = TEXT_SECONDARY;
        let primary = egui::Color32::from_rgb(220, 220, 225);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Exposure Time").size(12.0).color(secondary));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{:.1}\"", app.exposure_time))
                        .size(12.0)
                        .color(primary),
                );
            });
        });
        ui.spacing_mut().slider_width = ui.available_width();
        if ui
            .add(
                egui::Slider::new(&mut app.exposure_time, 0.001..=30.0)
                    .show_value(false)
                    .logarithmic(true),
            )
            .changed()
        {
            *changed = true;
        }
        ui.add_space(4.0);
    } else if labeled_slider(
        ui,
        "☀ Brightness",
        &mut app.exposure_time,
        0.001..=30.0,
        true,
    ) {
        *changed = true;
    }
    if labeled_slider(ui, "◑ Contrast", &mut app.gamma_boost, 0.5..=2.0, false) {
        *changed = true;
    }
    section_divider(ui);

    // Color
    section_header(ui, "COLOR");
    if labeled_slider(ui, "🔥 Warmth", &mut app.warmth, -1.0..=1.0, false) {
        *changed = true;
    }
    if labeled_slider(ui, "🌈 Intensity", &mut app.saturation, 0.0..=2.0, false) {
        *changed = true;
    }
    section_divider(ui);

    // Auto — same row
    ui.horizontal(|ui| {
        if ui.checkbox(&mut app.auto_levels, "🎚 Auto Levels").changed() {
            *changed = true;
        }
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("✨ Auto Enhance")
                        .size(11.0)
                        .color(TEXT_SECONDARY),
                )
                .fill(BG_MEDIUM)
                .stroke(egui::Stroke::NONE)
                .corner_radius(4.0),
            )
            .clicked()
        {
            app.white_balance_mode = filmr::WhiteBalanceMode::Auto;
            app.white_balance_strength = 1.0;
            *changed = true;
        }
    });
    section_divider(ui);

    // Professional-only: WB + Output
    if app.ux_mode == UxMode::Professional {
        professional::render_white_balance(app, ui, changed);
        section_divider(ui);
        professional::render_output_mode(app, ui, changed);
    }
}
