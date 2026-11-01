// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Theme settings and egui style mapping.

use egui::{Color32, Context, CornerRadius, FontId, Margin, Stroke, Style, TextStyle, Visuals};

use crate::tokens::{self, ThemeColors};

/// A Tallyline colour theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    /// Dark slate surfaces.
    #[default]
    Dark,
    /// Light surfaces.
    Light,
    /// High contrast surfaces.
    HighContrast,
}

/// Widget spacing preset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    /// Compact controls for data-dense screens.
    Compact,
    /// Comfortable controls for general screens.
    #[default]
    Comfortable,
}

/// Motion preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Motion {
    /// Use the design-system transitions.
    #[default]
    Full,
    /// Suppress transitions.
    Reduced,
}

/// Return resolved colours for a theme.
#[must_use]
pub(crate) const fn colors(theme: Theme) -> &'static ThemeColors {
    match theme {
        Theme::Dark => &tokens::DARK,
        Theme::Light => &tokens::LIGHT,
        Theme::HighContrast => &tokens::HIGH_CONTRAST,
    }
}

/// Apply Tallyline colours, typography and spacing to an egui context.
pub fn apply_theme(ctx: &Context, theme: Theme, density: Density, motion: Motion) {
    let palette = colors(theme);
    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(
        8.0,
        if density == Density::Compact {
            4.0
        } else {
            8.0
        },
    );
    style.spacing.button_padding = egui::vec2(
        12.0,
        if density == Density::Compact {
            6.0
        } else {
            9.0
        },
    );
    style.animation_time = if motion == Motion::Reduced { 0.0 } else { 0.12 };
    style.text_styles = [
        (TextStyle::Small, FontId::proportional(12.0)),
        (TextStyle::Body, FontId::proportional(14.0)),
        (TextStyle::Button, FontId::proportional(14.0)),
        (TextStyle::Heading, FontId::proportional(20.0)),
        (TextStyle::Monospace, FontId::monospace(13.0)),
    ]
    .into();
    let mut visuals = Visuals::dark();
    if theme == Theme::Light {
        visuals = Visuals::light();
    }
    visuals.override_text_color = Some(color(palette, "text-primary", Color32::WHITE));
    visuals.panel_fill = color(palette, "surface-1", visuals.panel_fill);
    visuals.window_fill = color(palette, "surface-2", visuals.window_fill);
    visuals.extreme_bg_color = color(palette, "surface-sunken", visuals.extreme_bg_color);
    visuals.faint_bg_color = color(palette, "surface-0", visuals.faint_bg_color);
    visuals.selection.bg_fill = color(palette, "selection-row-bg", visuals.selection.bg_fill);
    visuals.selection.stroke = Stroke::new(1.0, color(palette, "focus-ring", Color32::LIGHT_BLUE));
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);
    visuals.widgets.noninteractive.bg_stroke =
        Stroke::new(1.0, color(palette, "border-default", Color32::GRAY));
    visuals.widgets.inactive.bg_fill =
        color(palette, "surface-2", visuals.widgets.inactive.bg_fill);
    visuals.widgets.hovered.bg_fill = color(palette, "surface-3", visuals.widgets.hovered.bg_fill);
    visuals.widgets.active.bg_fill =
        color(palette, "accent-subtle-bg", visuals.widgets.active.bg_fill);
    style.visuals = visuals;
    style.spacing.window_margin = Margin::same(16);
    let egui_theme = if theme == Theme::Light {
        egui::Theme::Light
    } else {
        egui::Theme::Dark
    };
    ctx.set_style_of(egui_theme, style);
    ctx.set_theme(egui_theme);
}

fn color(palette: &ThemeColors, name: &str, fallback: Color32) -> Color32 {
    palette.color(name).unwrap_or(fallback)
}
