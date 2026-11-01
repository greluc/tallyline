// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Application preferences shared across public boundaries.

/// A supported Tallyline colour theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    /// Follow the operating system light or dark preference.
    #[default]
    System,
    /// Dark slate surfaces.
    Dark,
    /// Light surfaces.
    Light,
    /// High contrast surfaces.
    HighContrast,
}

/// User-owned application preferences.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppSettings {
    theme: Theme,
    onboarding_complete: bool,
}

impl AppSettings {
    /// Reconstructs persisted application preferences.
    #[must_use]
    pub const fn from_persisted(theme: Theme, onboarding_complete: bool) -> Self {
        Self {
            theme,
            onboarding_complete,
        }
    }

    /// Returns whether the start screen has been acknowledged.
    #[must_use]
    pub const fn onboarding_complete(&self) -> bool {
        self.onboarding_complete
    }
    /// Returns the selected colour theme.
    #[must_use]
    pub const fn theme(&self) -> Theme {
        self.theme
    }
}
