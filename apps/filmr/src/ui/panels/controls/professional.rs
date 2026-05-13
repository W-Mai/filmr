use filmr::light_leak::{LightLeak, LightLeakShape};
use filmr::{OutputMode, WhiteBalanceMode};

use crate::ui::app::{AppMode, FilmrApp};

use super::preset_io::create_custom_stock;
#[cfg(not(target_arch = "wasm32"))]
use super::preset_io::{export_preset, import_preset};
use crate::ui::components::{
    action_button, collapsing_section, labeled_slider, pill_selector, section_divider,
    section_header,
};

/// Effects tab: Lens + Light Leaks + Halation + Preset Management.
pub fn render_effects_tab(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    // Preset Management
    if app.mode == AppMode::Develop {
        collapsing_section(ui, "📦 Preset Management", false, |ui| {
            ui.horizontal(|ui| {
                if ui.add(action_button("Import")).clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    import_preset(app, changed);
                }
                if ui.add(action_button("Export")).clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    export_preset(app);
                }
                if ui.add(action_button("✨ Create Custom")).clicked() {
                    create_custom_stock(app, &ui.ctx().clone());
                    app.process_and_update_texture(&ui.ctx().clone());
                }
            });
            if app.selected_stock_idx >= app.builtin_stock_count
                && ui.add(action_button("📝 Edit in Studio")).clicked()
            {
                app.studio_stock = app.stocks[app.selected_stock_idx].as_ref().clone();
                app.studio_stock_idx = Some(app.selected_stock_idx);
                app.mode = AppMode::StockStudio;
                app.has_unsaved_changes = true;
                let ctx = ui.ctx().clone();
                app.process_and_update_texture(&ctx);
            }
        });
        ui.add_space(8.0);
    }

    section_header(ui, "LENS");
    if labeled_slider(
        ui,
        "Motion Blur",
        &mut app.motion_blur_amount,
        0.0..=3.0,
        false,
    ) {
        *changed = true;
    }
    if labeled_slider(
        ui,
        "Object Motion",
        &mut app.object_motion_amount,
        0.0..=2.0,
        false,
    ) {
        *changed = true;
    }
    if labeled_slider(ui, "DOF Amount", &mut app.dof_amount, 0.0..=2.0, false) {
        *changed = true;
    }
    if app.dof_amount > 0.0 {
        if labeled_slider(ui, "Focus Depth", &mut app.dof_focus, 0.0..=1.0, false) {
            *changed = true;
        }
        if labeled_slider(ui, "Swirly Bokeh", &mut app.dof_swirl, 0.0..=2.0, false) {
            *changed = true;
        }
    }
    if labeled_slider(
        ui,
        "Rotational Blur",
        &mut app.rotational_blur_amount,
        0.0..=2.0,
        false,
    ) {
        *changed = true;
    }
    section_divider(ui);

    // Light Leaks
    render_light_leaks(app, ui, changed);
    section_divider(ui);

    // Halation
    section_header(ui, "HALATION");
    if labeled_slider(ui, "Strength", &mut app.halation_strength, 0.0..=2.0, false) {
        *changed = true;
    }
    if labeled_slider(
        ui,
        "Threshold",
        &mut app.halation_threshold,
        0.0..=1.0,
        false,
    ) {
        *changed = true;
    }
    if labeled_slider(ui, "Spread", &mut app.halation_sigma, 0.0..=0.1, false) {
        *changed = true;
    }
}

/// Detail tab: Grain + Depth Map + Motion Trajectory.
pub fn render_detail_tab(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    // Grain
    section_header(ui, "GRAIN");
    if labeled_slider(ui, "Alpha", &mut app.grain_alpha, 0.0..=0.05, false) {
        *changed = true;
    }
    if labeled_slider(ui, "Sigma", &mut app.grain_sigma, 0.0..=0.05, false) {
        *changed = true;
    }
    if labeled_slider(ui, "Roughness", &mut app.grain_roughness, 0.0..=1.0, false) {
        *changed = true;
    }
    if labeled_slider(ui, "Blur", &mut app.grain_blur_radius, 0.0..=2.0, false) {
        *changed = true;
    }
    section_divider(ui);

    // Depth Map Preview
    if app.object_motion_amount > 0.0 || app.dof_amount > 0.0 {
        section_header(ui, "DEPTH MAP");
        if let Some(ref dm) = app.depth_map {
            ui.label("✅ Depth map ready");
            let preview_w = 160u32;
            let preview_h = (preview_w as f32 * dm.height as f32 / dm.width as f32) as u32;
            let size = egui::vec2(preview_w as f32, preview_h as f32);
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            let painter = ui.painter_at(rect);
            for py in (0..preview_h).step_by(2) {
                for px in (0..preview_w).step_by(2) {
                    let sx = (px as f32 / preview_w as f32 * dm.width as f32) as u32;
                    let sy = (py as f32 / preview_h as f32 * dm.height as f32) as u32;
                    let d = dm.get(sx, sy);
                    let r = ((1.0 - d) * 255.0) as u8;
                    let g = ((1.0 - d) * 200.0) as u8;
                    let b = ((1.0 - d * 0.5) * 255.0) as u8;
                    let pos = egui::pos2(rect.left() + px as f32, rect.top() + py as f32);
                    painter.rect_filled(
                        egui::Rect::from_min_size(pos, egui::vec2(2.0, 2.0)),
                        0.0,
                        egui::Color32::from_rgb(r, g, b),
                    );
                }
            }
        } else {
            ui.label("⚠ No depth map (model not found)");
        }
        ui.add_space(8.0);
    }

    // Motion Trajectory
    if app.motion_blur_amount > 0.0 {
        section_header(ui, "MOTION TRAJECTORY");
        ui.horizontal(|ui| {
            if ui
                .add(action_button("🎲"))
                .on_hover_text("New trajectory")
                .clicked()
            {
                app.motion_blur_seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                *changed = true;
            }
            ui.label(format!("seed: {}", app.motion_blur_seed));
        });

        let traj = filmr::shake::ShakeTrajectory::generate(
            app.motion_blur_amount * 25.0,
            64,
            app.motion_blur_seed,
        );

        ui.horizontal(|ui| {
            draw_trajectory_canvas(ui, &traj);
            draw_dwell_chart(ui, &traj);
        });
    }
}

/// White Balance section (Professional Adjust tab).
pub fn render_white_balance(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    section_header(ui, "WHITE BALANCE");

    let pre_wb = app.white_balance_mode;
    let options = [
        (WhiteBalanceMode::Auto, "Auto"),
        (WhiteBalanceMode::Gray, "Gray"),
        (WhiteBalanceMode::White, "White"),
        (WhiteBalanceMode::Off, "Off"),
    ];
    if pill_selector(ui, "wb_mode", &mut app.white_balance_mode, &options) {
        *changed = true;
    }

    ui.add_space(4.0);
    if app.white_balance_mode != WhiteBalanceMode::Off
        && labeled_slider(
            ui,
            "Strength",
            &mut app.white_balance_strength,
            0.0..=1.0,
            false,
        )
    {
        *changed = true;
    }

    if pre_wb != app.white_balance_mode {
        *changed = true;
    }
}

/// Output mode section — pill style.
pub fn render_output_mode(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    section_header(ui, "OUTPUT");
    let options = [
        (OutputMode::Positive, "Positive"),
        (OutputMode::Negative, "Negative"),
    ];
    if pill_selector(ui, "output_mode", &mut app.output_mode, &options) {
        *changed = true;
    }
}

fn render_light_leaks(app: &mut FilmrApp, ui: &mut egui::Ui, changed: &mut bool) {
    section_header(ui, "LIGHT LEAKS");

    if ui
        .checkbox(&mut app.light_leak_config.enabled, "Enable")
        .changed()
    {
        *changed = true;
    }
    if app.light_leak_config.enabled {
        ui.horizontal(|ui| {
            if ui.add(action_button("Add")).clicked() {
                app.light_leak_config.leaks.push(LightLeak::default());
                *changed = true;
            }
            if ui.add(action_button("Clear")).clicked() {
                app.light_leak_config.leaks.clear();
                *changed = true;
            }
        });

        let mut leaks_to_remove = Vec::new();
        for (i, leak) in app.light_leak_config.leaks.iter_mut().enumerate() {
            collapsing_section(ui, &format!("Leak #{}", i + 1), false, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Pos:");
                    if ui
                        .add(egui::Slider::new(&mut leak.position.0, 0.0..=1.0).text("X"))
                        .changed()
                    {
                        *changed = true;
                    }
                    if ui
                        .add(egui::Slider::new(&mut leak.position.1, 0.0..=1.0).text("Y"))
                        .changed()
                    {
                        *changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Color:");
                    if ui.color_edit_button_rgb(&mut leak.color).changed() {
                        *changed = true;
                    }
                });

                if ui
                    .add(egui::Slider::new(&mut leak.radius, 0.0..=1.5).text("Radius"))
                    .changed()
                {
                    *changed = true;
                }
                if ui
                    .add(egui::Slider::new(&mut leak.intensity, 0.0..=2.0).text("Intensity"))
                    .changed()
                {
                    *changed = true;
                }
                if ui
                    .add(
                        egui::Slider::new(&mut leak.rotation, 0.0..=std::f32::consts::TAU)
                            .text("Rotation"),
                    )
                    .changed()
                {
                    *changed = true;
                }
                if ui
                    .add(egui::Slider::new(&mut leak.roughness, 0.0..=1.0).text("Roughness"))
                    .changed()
                {
                    *changed = true;
                }

                egui::ComboBox::from_id_salt(format!("shape_{}", i))
                    .selected_text(format!("{:?}", leak.shape))
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(&mut leak.shape, LightLeakShape::Circle, "Circle")
                            .clicked()
                        {
                            *changed = true;
                        }
                        if ui
                            .selectable_value(&mut leak.shape, LightLeakShape::Linear, "Linear")
                            .clicked()
                        {
                            *changed = true;
                        }
                        if ui
                            .selectable_value(&mut leak.shape, LightLeakShape::Organic, "Organic")
                            .clicked()
                        {
                            *changed = true;
                        }
                        if ui
                            .selectable_value(&mut leak.shape, LightLeakShape::Plasma, "Plasma")
                            .clicked()
                        {
                            *changed = true;
                        }
                    });

                if ui.add(action_button("Remove")).clicked() {
                    leaks_to_remove.push(i);
                    *changed = true;
                }
            });
        }

        if !leaks_to_remove.is_empty() {
            for i in leaks_to_remove.into_iter().rev() {
                app.light_leak_config.leaks.remove(i);
            }
        }
    }
}

fn draw_trajectory_canvas(ui: &mut egui::Ui, traj: &filmr::shake::ShakeTrajectory) {
    let size = egui::vec2(120.0, 120.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    let cx = rect.center().x;
    let cy = rect.center().y;
    let pts: Vec<egui::Pos2> = traj
        .points
        .iter()
        .map(|&(x, y, _)| egui::pos2(cx + x, cy + y))
        .collect();

    if pts.len() >= 2 {
        for (i, w) in pts.windows(2).enumerate() {
            let weight = traj.points[i].2;
            let alpha = (weight * traj.points.len() as f32 * 255.0).clamp(30.0, 255.0) as u8;
            let stroke = egui::Stroke::new(
                1.5,
                egui::Color32::from_rgba_unmultiplied(255, 120, 60, alpha),
            );
            painter.line_segment([w[0], w[1]], stroke);
        }
        painter.circle_filled(pts[0], 3.0, egui::Color32::from_rgb(80, 220, 80));
        painter.circle_filled(
            *pts.last().unwrap(),
            3.0,
            egui::Color32::from_rgb(220, 80, 80),
        );
    }
}

fn draw_dwell_chart(ui: &mut egui::Ui, traj: &filmr::shake::ShakeTrajectory) {
    let size = egui::vec2(120.0, 120.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    let n = traj.points.len();
    if n <= 1 {
        return;
    }

    let max_w = traj.points.iter().map(|p| p.2).fold(0.0f32, f32::max);
    let bar_w = rect.width() / n as f32;

    for (i, &(_, _, w)) in traj.points.iter().enumerate() {
        let h = if max_w > 0.0 {
            w / max_w * (rect.height() - 4.0)
        } else {
            0.0
        };
        let x = rect.left() + i as f32 * bar_w;
        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(x, rect.bottom() - h),
            egui::vec2(bar_w.max(1.0), h),
        );
        let t = w / max_w.max(1e-6);
        let r = (t * 255.0) as u8;
        let b = ((1.0 - t) * 200.0) as u8;
        painter.rect_filled(bar_rect, 0.0, egui::Color32::from_rgb(r, 40, b));
    }

    painter.text(
        egui::pos2(rect.left() + 2.0, rect.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "dwell",
        egui::FontId::proportional(9.0),
        egui::Color32::from_gray(150),
    );
}
