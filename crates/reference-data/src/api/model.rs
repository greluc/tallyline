// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Immutable cached reference records and application state.

use tallyline_shared_kernel::api::model::{CommodityId, Timestamp};

pub use tallyline_shared_kernel::api::model::UexEnvironment;
/// A reference endpoint with an independent freshness window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endpoint {
    /// Commodity vocabulary and LIVE averages.
    Commodities,
}

impl Endpoint {
    /// Returns the endpoint's client freshness window in seconds.
    #[must_use]
    pub const fn ttl_seconds(self) -> u64 {
        match self {
            Self::Commodities => 3_600,
        }
    }
}

/// One commodity identified by its UEX ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commodity {
    id: CommodityId,
    name: String,
    code: String,
    available_live: bool,
    average_price: Option<u64>,
}

impl Commodity {
    /// Creates a boundary-mapped commodity.
    #[must_use]
    pub fn new(
        id: CommodityId,
        name: &str,
        code: &str,
        available_live: bool,
        average_price: Option<u64>,
    ) -> Self {
        Self {
            id,
            name: name.to_owned(),
            code: code.to_owned(),
            available_live,
            average_price,
        }
    }

    /// Returns the numeric UEX identity.
    #[must_use]
    pub const fn id(&self) -> CommodityId {
        self.id
    }

    /// Returns the display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the non-unique commodity code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Reports availability in the selected environment.
    #[must_use]
    pub const fn available(&self, environment: UexEnvironment) -> bool {
        match environment {
            UexEnvironment::Live => self.available_live,
            UexEnvironment::Ptu => true,
        }
    }

    /// Returns the LIVE average, with no substitute in PTU.
    #[must_use]
    pub const fn average_price(&self, environment: UexEnvironment) -> Option<u64> {
        match environment {
            UexEnvironment::Live => self.average_price,
            UexEnvironment::Ptu => None,
        }
    }
}

/// A successfully mapped response for one endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndpointData {
    /// Commodity vocabulary.
    Commodities(Vec<Commodity>),
}

/// One atomically persisted successful endpoint response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedReference {
    fetched_at: Timestamp,
    data: EndpointData,
}

impl CachedReference {
    /// Creates a successful response with its actual fetch time.
    #[must_use]
    pub const fn new(fetched_at: Timestamp, data: EndpointData) -> Self {
        Self { fetched_at, data }
    }

    /// Returns the response endpoint.
    #[must_use]
    pub const fn endpoint(&self) -> Endpoint {
        match self.data {
            EndpointData::Commodities(_) => Endpoint::Commodities,
        }
    }

    /// Returns the successful network fetch time.
    #[must_use]
    pub const fn fetched_at(&self) -> Timestamp {
        self.fetched_at
    }

    /// Returns the mapped response.
    #[must_use]
    pub const fn data(&self) -> &EndpointData {
        &self.data
    }
}

/// The visible reference-data connection state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceState {
    /// No successful cached reference response exists.
    NoReferenceData,
    /// Cached data is immediately usable.
    Cached,
    /// A refresh failed while retaining cached data.
    Offline,
}
