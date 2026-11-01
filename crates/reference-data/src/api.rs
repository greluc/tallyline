// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reference cache loading, searching and refresh commands.

pub mod model;

use model::{CachedReference, Commodity, Endpoint, EndpointData, ReferenceState, UexEnvironment};
use tallyline_shared_kernel::api::Clock;
use tallyline_shared_kernel::api::model::Timestamp;

/// Loads and atomically replaces successful reference responses.
pub trait ReferenceDataCache {
    /// The persistence failure type.
    type Error;

    /// Loads persisted responses without making a new network observation.
    ///
    /// # Errors
    /// Returns the storage or migration failure.
    fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error>;

    /// Replaces one response and its successful fetch time atomically.
    ///
    /// # Errors
    /// Returns a persistence error while preserving the previous response.
    fn replace(&mut self, entry: &CachedReference) -> Result<(), Self::Error>;
}

/// Serves cached reference data independently of background network work.
pub struct ReferenceApplication<S, C> {
    store: S,
    clock: C,
    entries: Vec<CachedReference>,
    state: ReferenceState,
}

impl<S: ReferenceDataCache, C: Clock> ReferenceApplication<S, C> {
    /// Loads cached data before any network work begins.
    ///
    /// # Errors
    /// Returns the cache's read or migration failure.
    pub fn load(mut store: S, clock: C) -> Result<Self, S::Error> {
        let entries = store.load()?;
        let state = if entries.is_empty() {
            ReferenceState::NoReferenceData
        } else {
            ReferenceState::Cached
        };
        Ok(Self {
            store,
            clock,
            entries,
            state,
        })
    }

    /// Searches names and non-unique codes without collapsing identities.
    #[must_use]
    pub fn commodities(&self, environment: UexEnvironment, query: &str) -> Vec<&Commodity> {
        let query = query.to_lowercase();
        self.entries
            .iter()
            .flat_map(|entry| match entry.data() {
                EndpointData::Commodities(items) => items.iter(),
            })
            .filter(|item| {
                item.available(environment)
                    && (item.name().to_lowercase().contains(&query)
                        || item.code().to_lowercase().contains(&query))
            })
            .collect()
    }

    /// Returns endpoints whose successful fetch has expired.
    #[must_use]
    pub fn due(&self) -> Vec<Endpoint> {
        [Endpoint::Commodities]
            .into_iter()
            .filter(|endpoint| {
                self.fetched_at(*endpoint).is_none_or(|at| {
                    self.clock.now().seconds().saturating_sub(at.seconds())
                        >= endpoint.ttl_seconds()
                })
            })
            .collect()
    }

    /// Publishes a successful network response only after persistence succeeds.
    ///
    /// # Errors
    /// Returns the storage failure without replacing active data or fetch time.
    pub fn refreshed(&mut self, data: EndpointData) -> Result<(), S::Error> {
        let entry = CachedReference::new(self.clock.now(), data);
        self.store.replace(&entry)?;
        self.entries
            .retain(|old| old.endpoint() != entry.endpoint());
        self.entries.push(entry);
        self.state = ReferenceState::Cached;
        Ok(())
    }
    /// Retains previous data and fetch times after a failed request.
    pub fn refresh_failed(&mut self, _endpoint: Endpoint) {
        if !self.entries.is_empty() {
            self.state = ReferenceState::Offline;
        }
    }

    /// Returns the visible connection state.
    #[must_use]
    pub const fn state(&self) -> ReferenceState {
        self.state
    }

    /// Returns an endpoint's last successful fetch time.
    #[must_use]
    pub fn fetched_at(&self, endpoint: Endpoint) -> Option<Timestamp> {
        self.entries
            .iter()
            .find(|entry| entry.endpoint() == endpoint)
            .map(CachedReference::fetched_at)
    }

    /// Returns the owned cache adapter.
    #[must_use]
    pub fn into_store(self) -> S {
        self.store
    }
}
