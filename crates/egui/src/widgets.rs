// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reusable Tallyline widgets.

use egui::{Color32, Response, RichText, Sense, Stroke, Ui, Widget, vec2};

use crate::{
    Icon,
    style::{Theme, colors},
};

/// A compact icon with a visible text label.
pub struct IconButton<'a> {
    /// Icon to display.
    pub icon: Icon,
    /// Button label.
    pub label: &'a str,
}

impl Widget for IconButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            self.icon
                .paint(ui.painter(), rect, ui.visuals().text_color());
            ui.button(self.label)
        })
        .inner
    }
}

/// A semantic status chip.
pub struct Chip<'a> {
    /// Chip content.
    pub text: &'a str,
    /// Foreground colour.
    pub foreground: Color32,
    /// Background colour.
    pub background: Color32,
}

impl Widget for Chip<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text = RichText::new(self.text).color(self.foreground);
        egui::Frame::new()
            .fill(self.background)
            .corner_radius(12)
            .inner_margin(egui::Margin::symmetric(9, 4))
            .show(ui, |ui| ui.label(text))
            .response
    }
}

/// A price deviation badge.
pub struct DeltaBadge<'a> {
    /// Signed delta label such as `Δ +12 %`.
    pub text: &'a str,
    /// Whether the deviation requires confirmation.
    pub major: bool,
    /// Theme used to resolve badge tokens.
    pub theme: Theme,
}

impl Widget for DeltaBadge<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let palette = colors(self.theme);
        let (background, foreground) = if self.major {
            (
                palette.color("deviation-major-badge-bg"),
                palette.color("deviation-major-badge-fg"),
            )
        } else {
            (None, palette.color("deviation-minor-fg"))
        };
        let foreground = foreground.unwrap_or(Color32::WHITE);
        let mut frame = egui::Frame::new()
            .corner_radius(3)
            .inner_margin(egui::Margin::symmetric(5, 2));
        if let Some(fill) = background {
            frame = frame.fill(fill);
        }
        frame
            .show(ui, |ui| {
                ui.label(RichText::new(self.text).color(foreground))
            })
            .response
    }
}

/// A bordered data field with optional supporting text.
pub struct Field<'a> {
    /// Field label.
    pub label: &'a str,
    /// Current field value.
    pub value: &'a str,
    /// Optional state explanation.
    pub reason: Option<&'a str>,
    /// Theme used to resolve field tokens.
    pub theme: Theme,
}

impl Widget for Field<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let colors = colors(self.theme);
        egui::Frame::new()
            .fill(
                colors
                    .color("surface-sunken")
                    .unwrap_or(Color32::TRANSPARENT),
            )
            .stroke(Stroke::new(
                1.0,
                colors.color("border-default").unwrap_or(Color32::GRAY),
            ))
            .corner_radius(4)
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.label(RichText::new(self.label).small().weak());
                ui.label(self.value);
                if let Some(reason) = self.reason {
                    ui.label(RichText::new(reason).small());
                }
            })
            .response
    }
}

/// A surface container with a subtle border.
pub struct Panel<'a> {
    /// Child content.
    pub content: &'a mut dyn FnMut(&mut Ui),
    /// Theme used to resolve panel tokens.
    pub theme: Theme,
}

impl Widget for Panel<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let palette = colors(self.theme);
        egui::Frame::new()
            .fill(
                palette
                    .color("surface-1")
                    .unwrap_or(ui.visuals().panel_fill),
            )
            .stroke(Stroke::new(
                1.0,
                palette.color("border-default").unwrap_or(Color32::GRAY),
            ))
            .corner_radius(6)
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| (self.content)(ui))
            .response
    }
}

/// A short-lived message with semantic tone.
pub struct Toast<'a> {
    /// Message content.
    pub text: &'a str,
    /// Semantic tone token prefix, for example `success` or `critical`.
    pub tone: &'a str,
    /// Theme used to resolve toast tokens.
    pub theme: Theme,
}

impl Widget for Toast<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let palette = colors(self.theme);
        let background = palette
            .color(&format!("{}-bg-strong", self.tone))
            .unwrap_or(Color32::DARK_GRAY);
        let foreground = palette
            .color(&format!("{}-fg-on-strong", self.tone))
            .unwrap_or(Color32::WHITE);
        egui::Frame::new()
            .fill(background)
            .corner_radius(5)
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.label(RichText::new(self.text).color(foreground))
            })
            .response
    }
}
