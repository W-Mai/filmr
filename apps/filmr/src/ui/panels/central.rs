use crate::ui::app::FilmrApp;
use egui::{Color32, Context, Pos2, Rect, RichText, Sense, Vec2};

pub fn render_central_panel(app: &mut FilmrApp, ctx: &Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // 1. Toolbar
        render_toolbar(app, ui, ctx);

        ui.separator();

        // 2. Image Canvas
        render_image_canvas(app, ui, ctx);

        // 3. Processing Overlay
        if app.is_processing || app.is_loading {
            render_processing_overlay(app, ui, ctx);
        }
    });
}

fn render_toolbar(app: &mut FilmrApp, ui: &mut egui::Ui, ctx: &Context) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);

        // Comparison Tools
        app.show_original = ui
            .add_sized([120.0, 32.0], egui::Button::new("👋 Hold to Compare"))
            .is_pointer_button_down_on();

        if ui
            .add_sized(
                [120.0, 32.0],
                egui::Button::new("🌓 Split View").selected(app.split_view),
            )
            .clicked()
        {
            app.split_view = !app.split_view;
        }

        ui.separator();

        // Actions
        if ui
            .add_sized([80.0, 32.0], egui::Button::new("🔬 Develop"))
            .clicked()
        {
            app.develop_image(ctx);
        }

        let save_btn = egui::Button::new("📄 Save").min_size(Vec2::new(80.0, 32.0));
        if ui
            .add_enabled(app.developed_image.is_some(), save_btn)
            .clicked()
        {
            app.save_image();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_sized(
                    Vec2::new(100.0, 32.0),
                    egui::Button::new("📊 Metrics").selected(app.show_metrics),
                )
                .clicked()
            {
                app.show_metrics = !app.show_metrics;
            }
            ui.add_space(10.0);
        });
    });
}

fn render_image_canvas(app: &mut FilmrApp, ui: &mut egui::Ui, ctx: &Context) {
    let rect = ui.available_rect_before_wrap();
    let response = ui.interact(rect, ui.id().with("image_area"), Sense::click_and_drag());

    // Handle Zoom & Pan
    let zoom_delta = ctx.input(|i| i.zoom_delta());
    if zoom_delta != 1.0 {
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
    if response.dragged() {
        app.offset += response.drag_delta();
    }
    if response.double_clicked() {
        app.zoom = 1.0;
        app.offset = Vec2::ZERO;
    }

    // Rendering
    if let Some(processed) = &app.processed_texture {
        let image_size = processed.size_vec2();
        let aspect = image_size.x / image_size.y;
        let view_aspect = rect.width() / rect.height();
        let base_scale = if aspect > view_aspect {
            rect.width() / image_size.x
        } else {
            rect.height() / image_size.y
        };
        let current_scale = base_scale * app.zoom;
        let new_size = image_size * current_scale;
        let center = rect.center() + app.offset;
        let image_rect = Rect::from_center_size(center, new_size);

        let painter = ui.painter_at(rect);

        if app.split_view && !app.show_original {
            if let Some(original) = &app.original_texture {
                // Render Original (Left side of split)
                let split_x = rect.min.x + rect.width() * app.split_pos;

                // Draw Original
                painter
                    .with_clip_rect(Rect::from_min_max(rect.min, Pos2::new(split_x, rect.max.y)))
                    .image(
                        original.id(),
                        image_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );

                // Draw Processed
                painter
                    .with_clip_rect(Rect::from_min_max(Pos2::new(split_x, rect.min.y), rect.max))
                    .image(
                        processed.id(),
                        image_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );

                // Split Line & Interaction
                let line_rect = Rect::from_center_size(
                    Pos2::new(split_x, rect.center().y),
                    Vec2::new(2.0, rect.height()),
                );
                painter.rect_filled(line_rect, 0.0, Color32::WHITE.gamma_multiply(0.5));

                let handle_rect = Rect::from_center_size(
                    Pos2::new(split_x, rect.center().y),
                    Vec2::new(20.0, 40.0),
                );
                let handle_res =
                    ui.interact(handle_rect, ui.id().with("split_handle"), Sense::drag());
                if handle_res.dragged() {
                    app.split_pos =
                        (handle_res.interact_pointer_pos().unwrap().x - rect.min.x) / rect.width();
                    app.split_pos = app.split_pos.clamp(0.0, 1.0);
                }
                if handle_res.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
            }
        } else {
            let texture = if app.show_original {
                app.original_texture.as_ref()
            } else {
                Some(processed)
            };
            if let Some(tex) = texture {
                painter.image(
                    tex.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }
    }
}

fn render_processing_overlay(_app: &mut FilmrApp, ui: &mut egui::Ui, ctx: &Context) {
    let rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_black_alpha(150));

    let spinner_chars = ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
    let index = (ctx.input(|i| i.time) * 10.0) as usize % spinner_chars.len();

    ui.put(
        rect,
        egui::Label::new(
            RichText::new(format!("{} Processing...", spinner_chars[index]))
                .color(Color32::WHITE)
                .strong()
                .size(24.0),
        ),
    );
    ctx.request_repaint();
}
