/// Dark professional theme inspired by Lightroom / Capture One / DaVinci Resolve.
use egui::{Color32, Stroke, Style, Visuals};

pub fn apply_dark_pro_theme(ctx: &egui::Context) {
    // Force dark mode, ignore system theme
    ctx.set_visuals(egui::Visuals::dark());

    let mut style = Style::default();

    let bg_dark = Color32::from_rgb(32, 32, 36);
    let bg_medium = Color32::from_rgb(42, 42, 48);
    let bg_hover = Color32::from_rgb(52, 52, 60);
    let bg_active = Color32::from_rgb(62, 62, 72);

    let text_primary = Color32::from_rgb(220, 220, 225);
    let text_secondary = Color32::from_rgb(150, 150, 160);

    let accent = Color32::from_rgb(230, 155, 50);

    let border = Color32::from_rgb(55, 55, 65);

    let mut visuals = Visuals::dark();

    // Panel backgrounds: side panels use bg_dark, central uses bg_darkest
    visuals.window_fill = bg_dark;
    visuals.window_stroke = Stroke::new(1.0, border);
    visuals.panel_fill = bg_dark;
    visuals.faint_bg_color = Color32::from_rgb(28, 28, 32);
    // DragValue/text edit bg matches panel — looks like plain text
    visuals.extreme_bg_color = bg_dark;

    // Noninteractive (labels, separators)
    visuals.widgets.noninteractive.bg_fill = bg_medium;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_secondary);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, border);
    visuals.widgets.noninteractive.corner_radius = 4.0.into();

    // Inactive (buttons, sliders at rest)
    visuals.widgets.inactive.bg_fill = bg_medium;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.corner_radius = 4.0.into();
    visuals.widgets.inactive.expansion = 0.0;

    // Hovered
    visuals.widgets.hovered.bg_fill = bg_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.corner_radius = 4.0.into();
    visuals.widgets.hovered.expansion = 1.0;

    // Active (pressed)
    visuals.widgets.active.bg_fill = bg_active;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.active.corner_radius = 4.0.into();
    visuals.widgets.active.expansion = 0.0;

    // Open (expanded combo boxes, etc.)
    visuals.widgets.open.bg_fill = bg_hover;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.open.corner_radius = 4.0.into();

    // Selection (selected items, slider fill)
    visuals.selection.bg_fill = accent.linear_multiply(0.5);
    visuals.selection.stroke = Stroke::new(1.5, accent);

    visuals.hyperlink_color = accent;
    visuals.override_text_color = Some(text_primary);
    visuals.warn_fg_color = Color32::from_rgb(255, 180, 50);
    visuals.error_fg_color = Color32::from_rgb(255, 80, 80);

    // Separator
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, border);

    // Slider handle — circle, accent colored
    visuals.handle_shape = egui::style::HandleShape::Circle;

    // Popup shadow
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(100),
    };

    // Window shadow
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };

    style.visuals = visuals;

    // Spacing — professional with breathing room
    style.spacing.item_spacing = egui::vec2(6.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.slider_width = 120.0;
    style.spacing.slider_rail_height = 4.0;
    style.spacing.scroll.bar_width = 5.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.indent = 12.0;
    style.spacing.interact_size = egui::vec2(40.0, 20.0); // comfortable click targets

    // Text styles — ark-pixel 12px base
    use egui::{FontId, TextStyle};
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(10.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(12.0));

    ctx.set_style(style);
}
