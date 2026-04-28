/// Dark professional theme inspired by Lightroom / Capture One / DaVinci Resolve.
use egui::{Color32, Stroke, Style, Visuals};

pub fn apply_dark_pro_theme(ctx: &egui::Context) {
    let mut style = Style::default();

    let bg_darkest = Color32::from_rgb(24, 24, 28);
    let bg_dark = Color32::from_rgb(32, 32, 36);
    let bg_medium = Color32::from_rgb(42, 42, 48);
    let bg_hover = Color32::from_rgb(52, 52, 60);
    let bg_active = Color32::from_rgb(62, 62, 72);

    let text_primary = Color32::from_rgb(220, 220, 225);
    let text_secondary = Color32::from_rgb(150, 150, 160);

    let accent = Color32::from_rgb(230, 155, 50);

    let border = Color32::from_rgb(55, 55, 65);
    let border_hover = Color32::from_rgb(80, 80, 95);

    let mut visuals = Visuals::dark();

    visuals.window_fill = bg_dark;
    visuals.window_stroke = Stroke::new(1.0, border);
    visuals.panel_fill = bg_darkest;
    visuals.faint_bg_color = bg_medium;
    visuals.extreme_bg_color = Color32::from_rgb(16, 16, 20);

    // Noninteractive
    visuals.widgets.noninteractive.bg_fill = bg_medium;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_secondary);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, border);

    // Inactive
    visuals.widgets.inactive.bg_fill = bg_medium;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.inactive.bg_stroke = Stroke::new(0.5, border);

    // Hovered
    visuals.widgets.hovered.bg_fill = bg_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, border_hover);

    // Active
    visuals.widgets.active.bg_fill = bg_active;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent);

    // Selection
    visuals.selection.bg_fill = accent.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, accent);

    visuals.hyperlink_color = accent;
    visuals.override_text_color = Some(text_primary);
    visuals.warn_fg_color = Color32::from_rgb(255, 180, 50);
    visuals.error_fg_color = Color32::from_rgb(255, 80, 80);

    style.visuals = visuals;

    // Spacing
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(12);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.slider_width = 180.0;
    style.spacing.scroll.bar_width = 6.0;

    // Text styles
    use egui::{FontId, TextStyle};
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(16.0));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(12.0));

    ctx.set_style(style);
}
