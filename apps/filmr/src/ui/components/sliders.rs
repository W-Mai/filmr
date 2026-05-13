//! Custom slider with label+value row above, full-width slider below.

use super::tokens::*;
use egui::{self, Layout, RichText};

/// Labeled slider: label+value top row (justify-between), full-width slider below.
/// Returns `true` if the value changed.
pub fn labeled_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    logarithmic: bool,
) -> bool {
    // Top row: label left, value right
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .size(BODY_FONT_SIZE)
                .color(TEXT_SECONDARY),
        );
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{:.2}", *value))
                    .size(BODY_FONT_SIZE)
                    .color(TEXT_PRIMARY),
            );
        });
    });

    // Slider full width, no text
    let mut slider = egui::Slider::new(value, range).show_value(false);
    if logarithmic {
        slider = slider.logarithmic(true);
    }
    ui.spacing_mut().slider_width = ui.available_width();
    let changed = ui.add(slider).changed();
    ui.add_space(4.0);
    changed
}

/// Section header — uppercase, muted color.
pub fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .strong()
            .size(BODY_FONT_SIZE)
            .color(TEXT_DISABLED),
    );
    ui.add_space(8.0);
}

/// Spacious divider between sections.
pub fn section_divider(ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
}

/// Custom collapsing section — PS layer-panel style.
/// Full-width clickable header with subtle background, no indent on content.
pub fn collapsing_section(
    ui: &mut egui::Ui,
    label: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let id = ui.id().with(label);
    let mut open = ui.memory_mut(|mem| *mem.data.get_temp_mut_or_insert_with(id, || default_open));

    // Header row — full width, subtle bg
    let desired_size = egui::vec2(ui.available_width(), 22.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // Background
    let bg = if resp.hovered() { BG_HOVER } else { BG_MEDIUM };
    ui.painter().rect_filled(rect, 2.0, bg);

    // Chevron + label
    let chevron = if open { "▾" } else { "▸" };
    let text_color = TEXT_SECONDARY;
    ui.painter().text(
        egui::pos2(rect.left() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        chevron,
        egui::FontId::proportional(10.0),
        text_color,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 22.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(BODY_FONT_SIZE),
        text_color,
    );

    if resp.clicked() {
        open = !open;
        ui.memory_mut(|mem| mem.data.insert_temp(id, open));
    }

    // Content — no indent, just vertical space
    if open {
        ui.add_space(4.0);
        add_contents(ui);
        ui.add_space(4.0);
    }
}
