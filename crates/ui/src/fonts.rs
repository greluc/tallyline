// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Unmodified static font resources approved by ADR-0004.

use egui::{FontData, FontDefinitions, FontFamily};

/// The approved font files loaded by the composition root.
pub struct FontFiles {
    /// IBM Plex Sans Regular.
    pub plex_regular: Vec<u8>,
    /// IBM Plex Sans SemiBold.
    pub plex_semibold: Vec<u8>,
    /// IBM Plex Sans Bold.
    pub plex_bold: Vec<u8>,
    /// JetBrains Mono NL Regular.
    pub mono_regular: Vec<u8>,
    /// JetBrains Mono NL Bold.
    pub mono_bold: Vec<u8>,
}

/// Installs the approved fonts from their separately shipped, unmodified files.
pub fn configure_fonts(context: &egui::Context, files: FontFiles) {
    let mut fonts = FontDefinitions::empty();
    for (name, bytes) in [
        ("IBM Plex Sans", files.plex_regular),
        ("IBM Plex Sans SemiBold", files.plex_semibold),
        ("IBM Plex Sans Bold", files.plex_bold),
        ("JetBrains Mono NL", files.mono_regular),
        ("JetBrains Mono NL Bold", files.mono_bold),
    ] {
        fonts
            .font_data
            .insert(name.to_owned(), FontData::from_owned(bytes).into());
    }
    fonts
        .families
        .insert(FontFamily::Proportional, vec!["IBM Plex Sans".to_owned()]);
    fonts
        .families
        .insert(FontFamily::Monospace, vec!["JetBrains Mono NL".to_owned()]);
    context.set_fonts(fonts);
}
