use crate::ui::app::FilmrApp;
use egui::Context;
use egui_uix::components::toggle::Toggle;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

pub fn render_settings_window(app: &mut FilmrApp, ctx: &Context) {
    let mut open = app.show_settings;
    egui::Window::new("⚙ Settings")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(400.0);
            if let Some(config_manager) = &mut app.config_manager {
                ui.heading("General");
                ui.separator();
                ui.add_space(5.0);

                ui.heading("Display");
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Smooth Histogram");
                        ui.add(Toggle::new(&mut app.hist_smooth, ""));
                    });
                });
                ui.add_space(5.0);

                ui.heading("Paths");
                ui.group(|ui| {
                    ui.label("Custom Films Directory:");
                    ui.horizontal(|ui| {
                        // Truncate path if too long
                        let path_str = config_manager.config.custom_stocks_path.to_string_lossy();
                        ui.label(
                            egui::RichText::new(&*path_str)
                                .monospace()
                                .background_color(ui.visuals().code_bg_color),
                        );

                        if ui.button("📂 Browse...").clicked() {
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(path) = FileDialog::new().pick_folder() {
                                config_manager.config.custom_stocks_path = path;
                                config_manager.save();
                            }
                        }
                    });
                    ui.label(
                        egui::RichText::new(
                            "Note: You may need to restart the app for changes to take effect.",
                        )
                        .weak()
                        .small(),
                    );
                });
            } else {
                ui.label("Config manager not available.");
            }

            ui.add_space(10.0);
            ui.separator();

            // Model Management
            #[cfg(feature = "depth")]
            {
                ui.heading("Depth Model");
                ui.group(|ui| {
                    if filmr::depth::is_model_available() {
                        ui.horizontal(|ui| {
                            ui.label("✅ Installed (~95MB)");
                            if ui.button("🗑 Delete").clicked() {
                                app.delete_model();
                            }
                        });
                    } else if let Some((dl, total)) = app.model_download_progress {
                        let pct = dl as f32 / total.max(1) as f32;
                        ui.add(egui::ProgressBar::new(pct).text(format!(
                            "{:.1} / {:.1} MB ({:.0}%)",
                            dl as f64 / 1e6,
                            total as f64 / 1e6,
                            pct * 100.0
                        )));
                    } else {
                        ui.label("❌ Not installed");
                        ui.label(
                            egui::RichText::new(
                                "Required for DOF, object motion, and depth preview.",
                            )
                            .weak()
                            .small(),
                        );
                        if let Some(ref err) = app.model_download_error {
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                        }
                        if ui.button("⬇ Download (~95MB)").clicked() {
                            app.start_model_download();
                        }
                    }
                });
            }

            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            });
        });
    app.show_settings = open;
}
