mod preset_io;
mod professional;
mod shutter_speed;
mod simple;

use egui::{Context, RichText};
use egui_uix::components::toggle::Toggle;

use crate::config::UxMode;
use crate::ui::app::FilmrApp;

use professional::render_professional_controls;
use simple::render_simple_controls;

pub use shutter_speed::ShutterSpeed;

pub fn render_controls(app: &mut FilmrApp, ctx: &Context) {
    let mut changed = false;
    egui::SidePanel::left("controls_panel").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                ui.heading(RichText::new("FILMR").strong().size(24.0));
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
            });
            ui.add_space(16.0);

            if app.ux_mode == UxMode::Simple {
                render_simple_controls(app, ui, ctx, &mut changed);
            } else {
                render_professional_controls(app, ui, ctx, &mut changed);
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.collapsing("⌨ Shortcuts & Help", |ui| {
                ui.small("- Drag & Drop image to open");
                ui.small("- Scroll/Pinch to Zoom");
                ui.small("- Drag image to Pan");
                ui.small("- Double Click to Reset View");
                ui.small("- Ctrl+O to Open File");
            });
        });

        egui::TopBottomPanel::bottom("ux_mode_panel").show(ctx, |ui| {
            // UX Mode Switcher
            ui.horizontal_centered(|ui| {
                ui.set_min_height(24.0);
                let prev_mode = app.ux_mode;
                let mut toggle_flag = app.ux_mode == UxMode::Professional;

                ui.label("👶");
                if ui
                    .add(Toggle::new(&mut toggle_flag, "🚀 Professional"))
                    .clicked()
                {
                    if let Some(cm) = &mut app.config_manager {
                        cm.config.ux_mode = app.ux_mode;
                        cm.save();
                    }
                }

                app.ux_mode = if toggle_flag {
                    UxMode::Professional
                } else {
                    UxMode::Simple
                };

                if prev_mode != app.ux_mode {
                    changed = true;
                }

                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("🔧").clicked() {
                            app.show_settings = true;
                        }
                    },
                );
            });
        });
    });

    if changed {
        app.process_and_update_texture(ctx);
        app.regenerate_thumbnails();
    }
}
