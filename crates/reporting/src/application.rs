// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Save-before-publish manual Report use cases.

use crate::api::model::{Report, ReportId, ReportKey, RowId};
use crate::api::{ReportRepository, RepositoryError};
use tallyline_game::api::model::EnvironmentMapping;
use tallyline_reference_data::api::model::ReferenceSnapshot;
use tallyline_shared_kernel::api::Clock;
use tallyline_shared_kernel::api::model::{
    CommodityId, Percent, PricePerScu, ScuQuantity, TradeSide,
};

/// An explicit user edit of a manual Draft.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManualEdit {
    /// Add an empty row from the available vocabulary.
    AddCommodity(CommodityId),
    /// Enter or clear one price.
    Price(RowId, Option<PricePerScu>),
    /// Enter or clear one inventory quantity.
    Stock(RowId, Option<ScuQuantity>),
    /// Select or clear one status code.
    Status(RowId, Option<std::num::NonZeroU8>),
    /// Explicitly take over this row's currently displayed reference values.
    TakeOver(RowId),
    /// Change terminal, side or explicitly selected environment.
    ChangeKey(ReportKey),
}

/// Manual Report use cases with injected persistence and time.
pub struct ReportingApplication<R, C> {
    repository: R,
    clock: C,
    mapping: EnvironmentMapping,
    price_tolerance_override: Option<Percent>,
    stock_settings: crate::api::model::StockSettings,
}
impl<R: ReportRepository, C: Clock> ReportingApplication<R, C> {
    /// Constructs the manual Draft application.
    pub fn new(repository: R, clock: C) -> Self {
        Self {
            repository,
            clock,
            mapping: EnvironmentMapping::default(),
            price_tolerance_override: None,
            stock_settings: crate::api::model::StockSettings::default(),
        }
    }
    /// Applies the currently persisted game-channel mapping.
    pub fn set_mapping(&mut self, mapping: EnvironmentMapping) {
        self.mapping = mapping;
    }
    /// Creates a persistent Draft before making it available to the editor.
    ///
    /// # Errors
    /// Returns invalid input or a repository failure.
    pub fn create(
        &mut self,
        key: ReportKey,
        game_version: Option<&str>,
    ) -> Result<Report, RepositoryError> {
        self.repository
            .create_manual(key, self.clock.now(), game_version)
    }
    /// Creates empty known-commodity rows against an explicitly chosen reference context.
    ///
    /// # Errors
    /// Refuses a mismatched environment, unavailable terminal or repository failure.
    pub fn create_from_reference(
        &mut self,
        key: ReportKey,
        snapshot: &ReferenceSnapshot,
    ) -> Result<Report, RepositoryError> {
        if self.mapping.environment(key.environment()) != snapshot.environment()
            || !snapshot
                .terminals("")
                .iter()
                .any(|terminal| terminal.id() == key.terminal())
        {
            return Err(RepositoryError::Invalid);
        }
        let version = snapshot
            .game_versions()
            .and_then(|versions| match snapshot.environment() {
                tallyline_shared_kernel::api::model::UexEnvironment::Live => {
                    Some(versions.live.as_str())
                }
                tallyline_shared_kernel::api::model::UexEnvironment::Ptu => versions.ptu.as_deref(),
            });
        let mut report = self.create(key, version)?;
        if let Some(priors) = snapshot.terminal_priors(key.terminal()) {
            for prior in priors
                .rows()
                .iter()
                .filter(|prior| prior.side() == key.side())
            {
                report
                    .add_commodity(prior.commodity())
                    .map_err(|_| RepositoryError::Invalid)?;
            }
        }
        if report.version() > 0 {
            self.repository.save_draft(&report, 0)?;
        }
        Ok(report)
    }
    /// Lists durable Drafts in most recently edited order.
    ///
    /// # Errors
    /// Returns invalid persisted state or unavailable storage.
    pub fn drafts(&mut self) -> Result<Vec<Report>, RepositoryError> {
        self.repository.drafts()
    }
    /// Validates and persists an explicit user edit against its expected version.
    ///
    /// # Errors
    /// Returns stale edits, invalid vocabulary/reference choices or unavailable storage.
    pub fn edit(
        &mut self,
        id: ReportId,
        expected: u64,
        edit: ManualEdit,
        snapshot: &ReferenceSnapshot,
    ) -> Result<Report, RepositoryError> {
        let mut report = self
            .repository
            .load_draft(id)?
            .ok_or(RepositoryError::Invalid)?;
        if report.version() != expected {
            return Err(RepositoryError::Conflict);
        }
        let key = match edit {
            ManualEdit::ChangeKey(key) => key,
            ManualEdit::AddCommodity(_)
            | ManualEdit::Price(..)
            | ManualEdit::Stock(..)
            | ManualEdit::Status(..)
            | ManualEdit::TakeOver(_) => report.key(),
        };
        if self.mapping.environment(key.environment()) != snapshot.environment() {
            return Err(RepositoryError::Invalid);
        }
        let result = match edit {
            ManualEdit::AddCommodity(commodity) => {
                let known = snapshot
                    .commodities("")
                    .iter()
                    .any(|item| item.id() == commodity)
                    || snapshot
                        .terminal_priors(key.terminal())
                        .is_some_and(|priors| {
                            priors
                                .rows()
                                .iter()
                                .any(|row| row.commodity() == commodity && row.side() == key.side())
                        });
                if !known {
                    return Err(RepositoryError::Invalid);
                }
                report.add_commodity(commodity).map(|_| ())
            }
            ManualEdit::Price(row, value) => report.set_price(row, value),
            ManualEdit::Stock(row, value) => report.set_stock(row, value),
            ManualEdit::Status(row, value) => {
                if let Some(value) = value {
                    let levels = snapshot.status_levels().ok_or(RepositoryError::Invalid)?;
                    let statuses = match key.side() {
                        TradeSide::Buy => &levels.buy,
                        TradeSide::Sell => &levels.sell,
                    };
                    if !statuses.iter().any(|level| level.code == value) {
                        return Err(RepositoryError::Invalid);
                    }
                }
                report.set_status(row, value)
            }
            ManualEdit::TakeOver(row) => {
                let commodity = report
                    .rows()
                    .iter()
                    .find(|item| item.id() == row)
                    .ok_or(RepositoryError::Invalid)?
                    .commodity();
                let prior = snapshot
                    .terminal_priors(key.terminal())
                    .and_then(|priors| {
                        priors.rows().iter().find(|prior| {
                            prior.commodity() == commodity && prior.side() == key.side()
                        })
                    })
                    .ok_or(RepositoryError::Invalid)?;
                let values = prior.values();
                report
                    .set_price(row, Some(values.price))
                    .and_then(|()| report.set_stock(row, Some(values.stock)))
                    .and_then(|()| report.set_status(row, values.status))
            }
            ManualEdit::ChangeKey(key) => {
                if !snapshot
                    .terminals("")
                    .iter()
                    .any(|terminal| terminal.id() == key.terminal())
                {
                    return Err(RepositoryError::Invalid);
                }
                report.change_key(key)
            }
        };
        result.map_err(|_| RepositoryError::Invalid)?;
        if report.version() > expected {
            self.repository.save_draft(&report, expected)?;
        }
        Ok(report)
    }
}

impl<R: ReportRepository, C: Clock> ReportingApplication<R, C> {
    /// Applies an optional tighten-only price override at the next assessment.
    pub fn set_price_tolerance_override(&mut self, value: Option<Percent>) {
        self.price_tolerance_override = value;
    }
}

mod prices;

impl<R: ReportRepository, C: Clock> ReportingApplication<R, C> {
    fn draft_for_reference(
        &mut self,
        id: ReportId,
        snapshot: &ReferenceSnapshot,
    ) -> Result<Report, RepositoryError> {
        let report = self
            .repository
            .load_draft(id)?
            .ok_or(RepositoryError::Invalid)?;
        if self.mapping.environment(report.key().environment()) != snapshot.environment() {
            return Err(RepositoryError::Invalid);
        }
        Ok(report)
    }
}

mod attachments;
mod review;
mod statuses;
mod stock;

impl<R: ReportRepository, C: Clock> ReportingApplication<R, C> {
    /// Applies validated shipping SCU bands at the next assessment.
    pub fn set_stock_settings(&mut self, settings: crate::api::model::StockSettings) {
        self.stock_settings = settings;
    }
}
