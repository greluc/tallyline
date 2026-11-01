// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rendering smoke check for every recreated state icon at common display scales.

use egui::{Color32, Pos2, RawInput, Rect, vec2};
use tallyline_egui::Icon;

const SCALES: [f32; 3] = [1.0, 1.25, 1.5];

#[test]
fn every_state_icon_paints_vector_shapes_without_font_fallback() {
    let context = egui::Context::default();
    for scale in SCALES {
        context.set_pixels_per_point(scale);
        let mut output = context.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(240.0, 80.0))),
                ..Default::default()
            },
            |ui| {
                for (index, icon) in Icon::STATE_ICONS.iter().copied().enumerate() {
                    let origin = Pos2::new(8.0 + index as f32 * 24.0, 8.0);
                    icon.paint(
                        ui.painter(),
                        Rect::from_min_size(origin, vec2(16.0, 16.0)),
                        Color32::WHITE,
                    );
                }
            },
        );
        // This test inspects vector shapes only; no renderer consumes the font texture updates.
        output.textures_delta.clear();
        assert!(output.shapes.len() > Icon::STATE_ICONS.len());
        assert!(
            output
                .shapes
                .iter()
                .all(|shape| !matches!(shape.shape, egui::Shape::Text(_)))
        );
    }
}
