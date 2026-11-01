// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Submission availability and bounded untrusted staff information.

use std::num::NonZeroU32;
use tallyline_shared_kernel::api::model::UexEnvironment;

const MAX_STAFF_NOTIFICATION_CHARACTERS: usize = 280;

/// A validated server acceptance switch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptanceSwitch {
    /// This switch permits reporting.
    Enabled,
    /// This switch suspends reporting.
    Disabled,
}
impl From<bool> for AcceptanceSwitch {
    fn from(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
}

/// Independent server switches that govern submissions without disabling browsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubmissionAcceptance {
    /// Datacenter service availability.
    pub datacenter: AcceptanceSwitch,
    /// LIVE reporting availability.
    pub reports: AcceptanceSwitch,
    /// Commodity reporting availability.
    pub commodity: AcceptanceSwitch,
    /// PTU reporting availability.
    pub ptu: AcceptanceSwitch,
}
impl SubmissionAcceptance {
    /// Reports whether this environment currently accepts commodity submissions.
    #[must_use]
    pub const fn allows(self, environment: UexEnvironment) -> bool {
        matches!(self.datacenter, AcceptanceSwitch::Enabled)
            && matches!(self.commodity, AcceptanceSwitch::Enabled)
            && match environment {
                UexEnvironment::Live => matches!(self.reports, AcceptanceSwitch::Enabled),
                UexEnvironment::Ptu => matches!(self.ptu, AcceptanceSwitch::Enabled),
            }
    }
}

/// Commodity-wide server tolerances and default prior staleness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommodityConstraints {
    /// Server price tolerance in whole percent.
    pub price_variation_percent: u32,
    /// Default staleness limit in days.
    pub prior_ttl_days: NonZeroU32,
    /// Server SCU tolerance for diagnostics, never a loosened client threshold.
    pub scu_variation_percent: u32,
}

/// Cached policy, constraints and capped literal staff text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceParameters {
    acceptance: SubmissionAcceptance,
    constraints: CommodityConstraints,
    notification: Option<String>,
}
impl ReferenceParameters {
    /// Retains validated parameters and at most 280 Unicode characters of literal staff text.
    #[must_use]
    pub fn new(
        acceptance: SubmissionAcceptance,
        constraints: CommodityConstraints,
        notification: Option<&str>,
    ) -> Self {
        Self {
            acceptance,
            constraints,
            notification: notification.map(|text| {
                text.chars()
                    .take(MAX_STAFF_NOTIFICATION_CHARACTERS)
                    .collect()
            }),
        }
    }
    /// Returns submission availability independently of vocabulary availability.
    #[must_use]
    pub const fn acceptance(&self) -> SubmissionAcceptance {
        self.acceptance
    }
    /// Returns server tolerances and prior staleness without applying them as client overrides.
    #[must_use]
    pub const fn constraints(&self) -> CommodityConstraints {
        self.constraints
    }
    /// Returns bounded untrusted text, requiring literal display without markup interpretation.
    #[must_use]
    pub fn staff_notification(&self) -> Option<&str> {
        self.notification.as_deref()
    }
}
