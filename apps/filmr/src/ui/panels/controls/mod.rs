mod preset_io;
mod professional;
mod shutter_speed;
mod simple;

use egui::{Context, RichText};

use crate::config::UxMode;
use crate::ui::app::{FilmrApp, RightTab};

pub use shutter_speed::ShutterSpeed;

/// Render left panel (film list + style) and right panel (adjustment tabs).
pub fn render_controls(app: &mut FilmrApp, ctx: &Context) {
    let mut changed = false;

    // ── Left Panel: Film List + Style ──
    egui::SidePanel::left("film_list_panel")
        .default_width(224.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(6.0);
                simple::render_film_list(app, ui, &mut changed);
                ui.add_space(8.0);
                simple::render_style_selector(app, ui, &mut changed);
                ui.add_space(8.0);
            });
        });

    // ── Right Panel: Adjustments ──
    egui::SidePanel::right("adjust_panel")
        .default_width(256.0)
        .show(ctx, |ui| {
            // Mode switch + tabs
            ui.horizontal(|ui| {
                let prev_mode = app.ux_mode;
                if ui
                    .selectable_label(app.ux_mode == UxMode::Simple, "Simple")
                    .clicked()
                {
                    app.ux_mode = UxMode::Simple;
                    app.right_tab = RightTab::Adjust;
                }
                if ui
                    .selectable_label(app.ux_mode == UxMode::Professional, "Professional")
                    .clicked()
                {
                    app.ux_mode = UxMode::Professional;
                }
                if prev_mode != app.ux_mode {
                    if let Some(cm) = &mut app.config_manager {
                        cm.config.ux_mode = app.ux_mode;
                        cm.save();
                    }
                    changed = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙").clicked() {
                        app.show_settings = true;
                    }
                });
            });
            ui.separator();

            // Tab bar (Professional only)
            if app.ux_mode == UxMode::Professional {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut app.right_tab, RightTab::Adjust, "Adjust");
                    ui.selectable_value(&mut app.right_tab, RightTab::Effects, "Effects");
                    ui.selectable_value(&mut app.right_tab, RightTab::Detail, "Detail");
                });
                ui.separator();
            }

            egui::ScrollArea::vertical().show(ui, |ui| match app.right_tab {
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

    if changed {
        app.process_and_update_texture(ctx);
        app.regenerate_thumbnails();
    }
}

/// Adjust tab — shown in both Simple and Professional modes.
fn render_adjust_tab(app: &mut FilmrApp, ui: &mut egui::Ui, _ctx: &Context, changed: &mut bool) {
    // Exposure
    ui.label(RichText::new("Exposure").strong());
    ui.add_space(2.0);
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
    } else {
        ui.horizontal(|ui| {
            ui.label("☀ Brightness");
        });
        if ui
            .add(egui::Slider::new(&mut app.exposure_time, 0.001..=30.0).logarithmic(true))
            .changed()
        {
            *changed = true;
        }
    }
    ui.horizontal(|ui| {
        ui.label("◑ Contrast");
    });
    if ui
        .add(egui::Slider::new(&mut app.gamma_boost, 0.5..=2.0))
        .changed()
    {
        *changed = true;
    }
    ui.add_space(8.0);

    // Color
    ui.label(RichText::new("Color").strong());
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.label("🔥 Warmth");
    });
    if ui
        .add(egui::Slider::new(&mut app.warmth, -1.0..=1.0))
        .changed()
    {
        *changed = true;
    }
    ui.horizontal(|ui| {
        ui.label("🌈 Intensity");
    });
    if ui
        .add(egui::Slider::new(&mut app.saturation, 0.0..=2.0))
        .changed()
    {
        *changed = true;
    }
    ui.add_space(8.0);

    // Auto
    if ui.checkbox(&mut app.auto_levels, "🎚 Auto Levels").changed() {
        *changed = true;
    }
    if ui
        .button(RichText::new("✨ Auto Enhance").strong())
        .clicked()
    {
        app.white_balance_mode = filmr::WhiteBalanceMode::Auto;
        app.white_balance_strength = 1.0;
        *changed = true;
    }
    ui.add_space(8.0);

    // Professional-only: WB + Output
    if app.ux_mode == UxMode::Professional {
        professional::render_white_balance(app, ui, changed);
        ui.add_space(8.0);
        professional::render_output_mode(app, ui, changed);
    }
}
