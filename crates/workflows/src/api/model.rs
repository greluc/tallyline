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
    stock_settings: tallyline_reporting::api::model::StockSettings,
    onboarding_complete: bool,
    update_checks: UpdateChecks,
    available_release: Option<ReleaseVersion>,
    dismissed_release: Option<ReleaseVersion>,
    last_update_attempt: Option<tallyline_shared_kernel::api::model::Timestamp>,
}

impl AppSettings {
    /// Reconstructs persisted application preferences.
    #[must_use]
    pub const fn from_persisted(theme: Theme, onboarding_complete: bool) -> Self {
        Self {
            theme,
            stock_settings: tallyline_reporting::api::model::StockSettings::baseline(),
            onboarding_complete,
            update_checks: UpdateChecks::Disabled,
            available_release: None,
            dismissed_release: None,
            last_update_attempt: None,
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

/// Permission to check the public release endpoint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UpdateChecks {
    /// No release requests are made.
    #[default]
    Disabled,
    /// At most one release request per 24 hours.
    Enabled,
}

impl AppSettings {
    /// Returns the persisted release-check permission.
    #[must_use]
    pub const fn update_checks(&self) -> UpdateChecks {
        self.update_checks
    }

    /// Returns the last reserved release request time, including failed requests.
    #[must_use]
    pub const fn last_update_attempt(
        &self,
    ) -> Option<tallyline_shared_kernel::api::model::Timestamp> {
        self.last_update_attempt
    }

    /// Reconstructs release-check preferences without changing other settings.
    #[must_use]
    pub const fn with_update_state(
        mut self,
        checks: UpdateChecks,
        attempt: Option<tallyline_shared_kernel::api::model::Timestamp>,
    ) -> Self {
        self.update_checks = checks;
        self.last_update_attempt = attempt;
        self
    }
}

/// A standard numeric release tag without prerelease or build metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReleaseVersion(u64, u64, u64);

impl ReleaseVersion {
    /// Parses the release process's exact `vMAJOR.MINOR.PATCH` format.
    #[must_use]
    pub fn parse(tag: &str) -> Option<Self> {
        let mut parts = tag.strip_prefix('v')?.split('.');
        let major = version_component(parts.next()?)?;
        let minor = version_component(parts.next()?)?;
        let patch = version_component(parts.next()?)?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self(major, minor, patch))
    }

    /// Returns the fixed release-page URL for this version.
    #[must_use]
    pub fn release_url(self) -> String {
        format!("https://github.com/greluc/tallyline/releases/tag/{self}")
    }
}

fn version_component(value: &str) -> Option<u64> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    value.parse().ok()
}

impl std::fmt::Display for ReleaseVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "v{}.{}.{}", self.0, self.1, self.2)
    }
}

/// The passive result of a release check.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum UpdateStatus {
    /// No check has completed in this session.
    #[default]
    NotChecked,
    /// The installed version is at least the latest standard release.
    Current,
    /// A newer standard release has an actionable release page.
    Available(ReleaseVersion),
    /// The known available release was explicitly dismissed.
    Dismissed,
    /// No answer could be validated; core work remains available.
    Unavailable,
}

/// A release request could not be completed or validated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpdateFailure;

impl AppSettings {
    /// Returns the last validated available release.
    #[must_use]
    pub const fn available_release(&self) -> Option<ReleaseVersion> {
        self.available_release
    }

    /// Returns the release whose notice was explicitly dismissed.
    #[must_use]
    pub const fn dismissed_release(&self) -> Option<ReleaseVersion> {
        self.dismissed_release
    }

    /// Restores only validated release identities and their dismissal choice.
    #[must_use]
    pub const fn with_release_notice(
        mut self,
        available: Option<ReleaseVersion>,
        dismissed: Option<ReleaseVersion>,
    ) -> Self {
        self.available_release = available;
        self.dismissed_release = dismissed;
        self
    }

    /// Restores passive notice state against the immutable installed version.
    #[must_use]
    pub fn cached_update(&self, installed: ReleaseVersion) -> UpdateStatus {
        if self.update_checks == UpdateChecks::Disabled {
            return UpdateStatus::NotChecked;
        }
        match self.available_release {
            Some(version) if version <= installed => UpdateStatus::Current,
            Some(version) if self.dismissed_release == Some(version) => UpdateStatus::Dismissed,
            Some(version) => UpdateStatus::Available(version),
            None => UpdateStatus::NotChecked,
        }
    }
}

impl AppSettings {
    /// Retains a validated available notice when a later check is unavailable.
    #[must_use]
    pub fn resolve_update(&self, installed: ReleaseVersion, result: UpdateStatus) -> UpdateStatus {
        match self.cached_update(installed) {
            UpdateStatus::NotChecked => result,
            cached @ (UpdateStatus::Available(_)
            | UpdateStatus::Dismissed
            | UpdateStatus::Current
            | UpdateStatus::Unavailable) => cached,
        }
    }
}

impl AppSettings {
    /// Returns independently validated SCU review bands.
    #[must_use]
    pub const fn stock_settings(&self) -> tallyline_reporting::api::model::StockSettings {
        self.stock_settings
    }
    /// Reconstructs preferences with validated SCU review bands.
    #[must_use]
    pub const fn with_stock_settings(
        mut self,
        settings: tallyline_reporting::api::model::StockSettings,
    ) -> Self {
        self.stock_settings = settings;
        self
    }
}
