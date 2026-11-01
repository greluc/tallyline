// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Small scalable state icons drawn with egui primitives.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind};

/// A state icon used by Tallyline widgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    /// Help or confirmation needed.
    Help,
    /// Choose between alternatives.
    AltRoute,
    /// Correct a value.
    EditSquare,
    /// User-entered value.
    Edit,
    /// Confirmed value.
    CheckCircle,
    /// Double confirmation.
    DoneAll,
    /// Compare values.
    CompareArrows,
    /// Critical warning.
    CrisisAlert,
    /// New item without a reference.
    FiberNew,
}

/// A drawable glyph and its accessible name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconGlyph {
    /// The icon kind.
    pub icon: Icon,
    /// Localized or user-facing accessible label.
    pub label: &'static str,
}

impl Icon {
    /// The nine icons used for confidence and deviation states.
    pub const STATE_ICONS: [Self; 9] = [
        Self::Help,
        Self::AltRoute,
        Self::EditSquare,
        Self::Edit,
        Self::CheckCircle,
        Self::DoneAll,
        Self::CompareArrows,
        Self::CrisisAlert,
        Self::FiberNew,
    ];

    /// Draw this icon centered in `rect`.
    pub fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        let stroke = Stroke::new((rect.width() / 12.0).clamp(1.0, 2.0), color);
        let c = rect.center();
        let r = rect.width().min(rect.height()) * 0.36;
        match self {
            Self::Help => {
                painter.circle_stroke(c, r, stroke);
                painter.add(egui::Shape::line(
                    vec![
                        Pos2::new(c.x - r * 0.24, c.y - r * 0.12),
                        Pos2::new(c.x - r * 0.12, c.y - r * 0.35),
                        Pos2::new(c.x + r * 0.2, c.y - r * 0.32),
                        Pos2::new(c.x + r * 0.24, c.y - r * 0.06),
                        Pos2::new(c.x, c.y + r * 0.12),
                        Pos2::new(c.x, c.y + r * 0.28),
                    ],
                    stroke,
                ));
                painter.circle_filled(Pos2::new(c.x, c.y + r * 0.58), stroke.width * 0.7, color);
            }
            Self::AltRoute => {
                painter.line_segment(
                    [
                        Pos2::new(rect.left() + r * 0.2, c.y),
                        Pos2::new(rect.right() - r * 0.2, c.y),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [Pos2::new(c.x, c.y), Pos2::new(c.x + r * 0.6, c.y - r * 0.6)],
                    stroke,
                );
                painter.line_segment(
                    [Pos2::new(c.x, c.y), Pos2::new(c.x + r * 0.6, c.y + r * 0.6)],
                    stroke,
                );
            }
            Self::EditSquare => {
                painter.rect_stroke(rect.shrink(r * 0.2), 2, stroke, StrokeKind::Inside);
                painter.line_segment(
                    [
                        Pos2::new(c.x - r * 0.4, c.y + r * 0.4),
                        Pos2::new(c.x + r * 0.45, c.y - r * 0.45),
                    ],
                    stroke,
                );
            }
            Self::Edit => {
                painter.line_segment(
                    [
                        Pos2::new(c.x - r * 0.6, c.y + r * 0.6),
                        Pos2::new(c.x + r * 0.55, c.y - r * 0.55),
                    ],
                    stroke,
                );
            }
            Self::CheckCircle => {
                painter.circle_stroke(c, r, stroke);
                painter.line_segment(
                    [
                        Pos2::new(c.x - r * 0.5, c.y),
                        Pos2::new(c.x - r * 0.1, c.y + r * 0.4),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(c.x - r * 0.1, c.y + r * 0.4),
                        Pos2::new(c.x + r * 0.55, c.y - r * 0.45),
                    ],
                    stroke,
                );
            }
            Self::DoneAll => {
                painter.line_segment(
                    [
                        Pos2::new(c.x - r * 0.8, c.y),
                        Pos2::new(c.x - r * 0.2, c.y + r * 0.45),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(c.x - r * 0.2, c.y + r * 0.45),
                        Pos2::new(c.x + r * 0.7, c.y - r * 0.5),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(c.x - r * 0.1, c.y),
                        Pos2::new(c.x + r * 0.5, c.y + r * 0.45),
                    ],
                    stroke,
                );
            }
            Self::CompareArrows => {
                painter.line_segment(
                    [
                        Pos2::new(c.x - r, c.y - r * 0.35),
                        Pos2::new(c.x + r, c.y - r * 0.35),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(c.x + r, c.y - r * 0.35),
                        Pos2::new(c.x + r * 0.55, c.y - r * 0.75),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(c.x + r, c.y - r * 0.35),
                        Pos2::new(c.x + r * 0.55, c.y + r * 0.05),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(c.x + r, c.y + r * 0.35),
                        Pos2::new(c.x - r, c.y + r * 0.35),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(c.x - r, c.y + r * 0.35),
                        Pos2::new(c.x - r * 0.55, c.y - r * 0.05),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(c.x - r, c.y + r * 0.35),
                        Pos2::new(c.x - r * 0.55, c.y + r * 0.75),
                    ],
                    stroke,
                );
            }
            Self::CrisisAlert => {
                let points = [
                    Pos2::new(c.x, c.y - r),
                    Pos2::new(c.x + r, c.y + r * 0.8),
                    Pos2::new(c.x - r, c.y + r * 0.8),
                ];
                painter.add(egui::Shape::closed_line(points.to_vec(), stroke));
                painter.line_segment(
                    [
                        Pos2::new(c.x, c.y - r * 0.35),
                        Pos2::new(c.x, c.y + r * 0.25),
                    ],
                    stroke,
                );
                painter.circle_filled(Pos2::new(c.x, c.y + r * 0.55), 1.0, color);
            }
            Self::FiberNew => {
                painter.circle_filled(c, r, color);
                let plus = Stroke::new(stroke.width, Color32::BLACK);
                painter.line_segment(
                    [Pos2::new(c.x - r * 0.5, c.y), Pos2::new(c.x + r * 0.5, c.y)],
                    plus,
                );
                painter.line_segment(
                    [Pos2::new(c.x, c.y - r * 0.5), Pos2::new(c.x, c.y + r * 0.5)],
                    plus,
                );
            }
        }
    }
}
