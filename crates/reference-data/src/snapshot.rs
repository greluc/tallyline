// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Owned environment-specific read state independent of cache workers.

use crate::api::model::{
    CachedReference, Commodity, CommodityAverages, Endpoint, EndpointData, GameVersions,
    ReferenceState, StatusLevels, Terminal, UexEnvironment,
};
use tallyline_shared_kernel::api::model::{Timestamp, TradeSide};

/// Immutable reference data for one mapped UEX environment and one decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSnapshot {
    environment: UexEnvironment,
    state: ReferenceState,
    entries: Vec<CachedReference>,
}

impl ReferenceSnapshot {
    pub(crate) fn new(
        environment: UexEnvironment,
        state: ReferenceState,
        entries: &[CachedReference],
    ) -> Self {
        let entries = entries
            .iter()
            .filter(|entry| {
                environment != UexEnvironment::Ptu
                    || !matches!(entry.data(), EndpointData::TerminalPriors(_))
            })
            .map(|entry| match entry.data() {
                EndpointData::Commodities(items) => CachedReference::new(
                    entry.fetched_at(),
                    EndpointData::Commodities(
                        items
                            .iter()
                            .filter(|item| item.available(environment))
                            .map(|item| {
                                Commodity::new(
                                    item.id(),
                                    item.name(),
                                    item.code(),
                                    item.available(UexEnvironment::Live),
                                    CommodityAverages::new(
                                        item.average_price(environment, TradeSide::Buy),
                                        item.average_price(environment, TradeSide::Sell),
                                    ),
                                )
                            })
                            .collect(),
                    ),
                ),
                EndpointData::Terminals(_)
                | EndpointData::GameVersions(_)
                | EndpointData::StatusLevels(_)
                | EndpointData::Parameters(_)
                | EndpointData::TerminalPriors(_) => entry.clone(),
            })
            .collect();
        Self {
            environment,
            state,
            entries,
        }
    }

    /// Returns the mapped UEX environment, independent of the launcher's channel name.
    #[must_use]
    pub const fn environment(&self) -> UexEnvironment {
        self.environment
    }

    /// Returns matching commodities with no LIVE average retained in a PTU snapshot.
    #[must_use]
    pub fn commodities(&self, query: &str) -> Vec<&Commodity> {
        let query = query.to_lowercase();
        self.entries
            .iter()
            .flat_map(|entry| match entry.data() {
                EndpointData::Commodities(items) => items.as_slice(),
                EndpointData::Terminals(_)
                | EndpointData::GameVersions(_)
                | EndpointData::StatusLevels(_)
                | EndpointData::Parameters(_)
                | EndpointData::TerminalPriors(_) => &[],
            })
            .filter(|item| {
                item.name().to_lowercase().contains(&query)
                    || item.code().to_lowercase().contains(&query)
            })
            .collect()
    }

    /// Searches selectable terminal labels and location context for this environment.
    #[must_use]
    pub fn terminals(&self, query: &str) -> Vec<&Terminal> {
        let query = query.to_lowercase();
        self.entries
            .iter()
            .flat_map(|entry| match entry.data() {
                EndpointData::Terminals(items) => items.as_slice(),
                EndpointData::Commodities(_)
                | EndpointData::GameVersions(_)
                | EndpointData::StatusLevels(_)
                | EndpointData::Parameters(_)
                | EndpointData::TerminalPriors(_) => &[],
            })
            .filter(|terminal| {
                let text = terminal.text();
                terminal.available(self.environment)
                    && [
                        &text.name,
                        &text.nickname,
                        &text.display_name,
                        &text.location,
                    ]
                    .iter()
                    .any(|field| field.to_lowercase().contains(&query))
            })
            .collect()
    }

    /// Returns cached version read data, never a fresh network observation.
    #[must_use]
    pub fn game_versions(&self) -> Option<&GameVersions> {
        self.entries.iter().find_map(|entry| match entry.data() {
            EndpointData::GameVersions(versions) => Some(versions),
            EndpointData::Commodities(_)
            | EndpointData::Terminals(_)
            | EndpointData::StatusLevels(_)
            | EndpointData::Parameters(_)
            | EndpointData::TerminalPriors(_) => None,
        })
    }

    /// Returns the cached per-side status vocabulary.
    #[must_use]
    pub fn status_levels(&self) -> Option<&StatusLevels> {
        self.entries.iter().find_map(|entry| match entry.data() {
            EndpointData::StatusLevels(levels) => Some(levels),
            EndpointData::Commodities(_)
            | EndpointData::Terminals(_)
            | EndpointData::GameVersions(_)
            | EndpointData::Parameters(_)
            | EndpointData::TerminalPriors(_) => None,
        })
    }
    /// Returns cached submission policy, tolerances and capped literal staff text.
    #[must_use]
    pub fn parameters(&self) -> Option<&crate::api::model::ReferenceParameters> {
        self.entries.iter().find_map(|entry| match entry.data() {
            EndpointData::Parameters(parameters) => Some(parameters),
            EndpointData::Commodities(_)
            | EndpointData::Terminals(_)
            | EndpointData::GameVersions(_)
            | EndpointData::StatusLevels(_)
            | EndpointData::TerminalPriors(_) => None,
        })
    }

    /// Returns the LIVE assortment for one terminal, with no LIVE prior retained in PTU.
    #[must_use]
    pub fn terminal_priors(
        &self,
        terminal: tallyline_shared_kernel::api::model::TerminalId,
    ) -> Option<&crate::api::model::TerminalPriors> {
        self.entries.iter().find_map(|entry| match entry.data() {
            EndpointData::TerminalPriors(priors) if priors.terminal() == terminal => Some(priors),
            EndpointData::Commodities(_)
            | EndpointData::Terminals(_)
            | EndpointData::GameVersions(_)
            | EndpointData::StatusLevels(_)
            | EndpointData::Parameters(_)
            | EndpointData::TerminalPriors(_) => None,
        })
    }

    /// Returns one endpoint's successful fetch time without creating an observation.
    #[must_use]
    pub fn fetched_at(&self, endpoint: Endpoint) -> Option<Timestamp> {
        self.entries
            .iter()
            .find(|entry| entry.endpoint() == endpoint)
            .map(CachedReference::fetched_at)
    }

    /// Returns the connection state captured with these records.
    #[must_use]
    pub const fn state(&self) -> ReferenceState {
        self.state
    }
}
