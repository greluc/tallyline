// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Filesystem adapter results.

pub use crate::config::{LoadedSettings, SettingsIssue};
pub use crate::layout::DirectoryIssue;

/// The RGBA pixels of immutable bundled legal artwork.
pub struct LogoRaster {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Unpremultiplied red, green, blue and alpha bytes in row order.
    pub rgba: Vec<u8>,
}
pub use crate::game_config::LoadedGamePreferences;
