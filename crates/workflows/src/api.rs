// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Application settings workflow and persistence port.

pub mod model;

use model::{AppSettings, Theme};

/// Persists application preferences as one atomic update.
pub trait AppSettingsStore {
    /// A persistence failure.
    type Error;

    /// Saves preferences without changing the previous version on failure.
    ///
    /// # Errors
    /// Returns the storage failure when the update cannot be committed.
    fn save(&mut self, settings: &AppSettings) -> Result<(), Self::Error>;
}

/// Applies settings only after their persistence succeeds.
pub struct SettingsApplication<S> {
    settings: AppSettings,
    store: S,
}

impl<S: AppSettingsStore> SettingsApplication<S> {
    /// Creates the workflow from loaded preferences and their store.
    #[must_use]
    pub const fn new(settings: AppSettings, store: S) -> Self {
        Self { settings, store }
    }

    /// Returns the active preferences.
    #[must_use]
    pub const fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Persists and applies the selected theme.
    ///
    /// # Errors
    /// Returns a storage error and keeps the active preferences unchanged.
    pub fn change_theme(&mut self, theme: Theme) -> Result<(), S::Error> {
        let next = AppSettings::from_persisted(theme, self.settings.onboarding_complete());
        self.store.save(&next)?;
        self.settings = next;
        Ok(())
    }

    /// Persists acknowledgement of the onboarding start screen.
    ///
    /// # Errors
    /// Returns a storage error and keeps onboarding incomplete.
    pub fn complete_onboarding(&mut self) -> Result<(), S::Error> {
        let next = AppSettings::from_persisted(self.settings.theme(), true);
        self.store.save(&next)?;
        self.settings = next;
        Ok(())
    }

    /// Returns the owned settings store.
    #[must_use]
    pub fn into_store(self) -> S {
        self.store
    }
}
