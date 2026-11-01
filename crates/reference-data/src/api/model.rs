// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Immutable cached reference records and application state.

use tallyline_shared_kernel::api::model::{CommodityId, TerminalId, Timestamp};

pub use crate::snapshot::ReferenceSnapshot;
pub use tallyline_shared_kernel::api::model::UexEnvironment;
/// A reference endpoint with an independent freshness window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endpoint {
    /// Commodity vocabulary and LIVE averages.
    Commodities,
    /// Selectable commodity terminal vocabulary.
    Terminals,
    /// Current LIVE and optional PTU versions.
    GameVersions,
    /// Per-side inventory status labels.
    StatusLevels,
}

impl Endpoint {
    /// Returns the endpoint's client freshness window in seconds.
    #[must_use]
    pub const fn ttl_seconds(self) -> u64 {
        match self {
            Self::Commodities => 3_600,
            Self::Terminals => 43_200,
            Self::GameVersions | Self::StatusLevels => 86_400,
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
    /// Selectable commodity terminals.
    Terminals(Vec<Terminal>),
    /// Current versions returned by a successful public request.
    GameVersions(GameVersions),
    /// Current per-side inventory status vocabulary.
    StatusLevels(StatusLevels),
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
            EndpointData::Terminals(_) => Endpoint::Terminals,
            EndpointData::GameVersions(_) => Endpoint::GameVersions,
            EndpointData::StatusLevels(_) => Endpoint::StatusLevels,
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

/// Decoded terminal names and location context, none of which is a unique key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalText {
    /// The full terminal name.
    pub name: String,
    /// The terminal's abbreviated name.
    pub nickname: String,
    /// The API's display label.
    pub display_name: String,
    /// Available location names, retained for disambiguation.
    pub location: String,
}

/// One selectable commodity terminal identified by its UEX ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terminal {
    id: TerminalId,
    text: TerminalText,
    available_live: bool,
}

impl Terminal {
    /// Creates a terminal after the adapter excludes unsupported terminal types.
    #[must_use]
    pub const fn new(id: TerminalId, text: TerminalText, available_live: bool) -> Self {
        Self {
            id,
            text,
            available_live,
        }
    }

    /// Returns the numeric UEX identity.
    #[must_use]
    pub const fn id(&self) -> TerminalId {
        self.id
    }

    /// Returns decoded display and search text.
    #[must_use]
    pub const fn text(&self) -> &TerminalText {
        &self.text
    }

    /// Reports availability without applying LIVE flags to PTU.
    #[must_use]
    pub const fn available(&self, environment: UexEnvironment) -> bool {
        match environment {
            UexEnvironment::Live => self.available_live,
            UexEnvironment::Ptu => true,
        }
    }
}

/// Current UEX version strings, preserved without imposing semantic versioning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameVersions {
    /// The current LIVE version.
    pub live: String,
    /// The current PTU version, absent while none is published.
    pub ptu: Option<String>,
}

/// A persisted successful network response, never produced by cache loading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceDataRefreshed {
    /// The endpoint actually fetched.
    pub endpoint: Endpoint,
    /// The successful network observation time.
    pub fetched_at: Timestamp,
    /// A new version-history observation only for a game-versions fetch.
    pub game_versions: Option<GameVersions>,
}

/// One non-zero inventory status code with its side-specific labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLevel {
    /// The server's non-zero code, whose membership is checked against this vocabulary.
    pub code: std::num::NonZeroU8,
    /// The complete display label.
    pub name: String,
    /// The abbreviated display label.
    pub short_name: String,
    /// The server's abbreviation.
    pub abbreviation: String,
}

/// Independent buy and sell inventory-status vocabularies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLevels {
    /// Buy-side labels in server order.
    pub buy: Vec<StatusLevel>,
    /// Sell-side labels in server order.
    pub sell: Vec<StatusLevel>,
}
