// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tallyline design tokens and reusable widgets for egui.

#![forbid(unsafe_code)]

mod icon;
mod style;
mod widgets;

pub mod tokens;

pub use icon::{Icon, IconGlyph};
pub use style::{Density, Motion, Theme, apply_theme};
pub use widgets::{Chip, DeltaBadge, Field, IconButton, Panel, Toast};
