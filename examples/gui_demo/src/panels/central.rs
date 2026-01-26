use crate::app::FilmrApp;
use egui::{Context, Pos2, Rect, Sense, Vec2};

pub fn render_central_panel(app: &mut FilmrApp, ctx: &Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Toolbar Overlay
        ui.horizontal(|ui| {
            // Hold to Compare (Larger and Conspicuous)
            app.show_original = ui
                .add_sized(
                    [150.0, 40.0],
                    egui::Button::new("HOLD TO COMPARE").min_size(Vec2::new(150.0, 40.0)),
                )
                .is_pointer_button_down_on();

            ui.separator();

            if ui
                .add_sized([100.0, 40.0], egui::Button::new("Develop"))
                .clicked()
            {
                app.develop_image(ctx);
            }

            let save_btn = egui::Button::new("Save").min_size(Vec2::new(100.0, 40.0));
            if ui
                .add_enabled(app.developed_image.is_some(), save_btn)
                .clicked()
            {
                app.save_image();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let metrics_button = egui::Button::new("Metrics Panel").selected(app.show_metrics);
                let response = ui.add_sized(Vec2::new(100.0, 40.0), metrics_button);
                if response.clicked() {
                    app.show_metrics = !app.show_metrics;
                }

                ui.separator();
            });
        });
        ui.separator();

        let texture_to_show = if app.show_original {
            app.original_texture.as_ref()
        } else {
            app.processed_texture.as_ref()
        };

        if app.is_processing {
            ui.centered_and_justified(|ui| {
                ui.spinner();
            });
        }

        if let Some(texture) = texture_to_show {
            // Interactive Area
            let rect = ui.available_rect_before_wrap();
            let response = ui.interact(rect, ui.id().with("image_area"), Sense::click_and_drag());

            // 1. Handle Zoom (Pinch or Ctrl+Scroll)
            let zoom_delta = ctx.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                // Zoom towards mouse pointer
                if let Some(pointer_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    let center = rect.center();
                    let pointer_in_layer = pointer_pos - center;
                    let offset_to_pointer = pointer_in_layer - app.offset;

                    app.offset -= offset_to_pointer * (zoom_delta - 1.0);
                    app.zoom *= zoom_delta;
                } else {
                    app.zoom *= zoom_delta;
                }
            }

            // 2. Handle Pan (Drag)
            if response.dragged() {
                app.offset += response.drag_delta();
            }

            // 3. Double Click to Reset
            if response.double_clicked() {
                app.zoom = 1.0;
                app.offset = Vec2::ZERO;
            }

            // 4. Draw Image
            let image_size = texture.size_vec2();
            let aspect = image_size.x / image_size.y;
            let view_aspect = rect.width() / rect.height();

            let base_scale = if aspect > view_aspect {
                rect.width() / image_size.x
            } else {
                rect.height() / image_size.y
            };

            let current_scale = base_scale * app.zoom;
            let displayed_size = image_size * current_scale;

            let center = rect.center() + app.offset;
            let image_rect = Rect::from_center_size(center, displayed_size);

            let painter = ui.painter_at(rect);
            painter.image(
                texture.id(),
                image_rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Drag and drop an image file here");
            });
        }
    });
}
