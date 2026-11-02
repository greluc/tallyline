// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Parameterised SCU assessment without an implicit calibrated default.

use super::ReportingApplication;
use crate::api::model::{ReportId, StockAssessment};
use crate::api::{ReportRepository, RepositoryError};
use tallyline_reference_data::api::model::{ReferenceSnapshot, ScuTolerance};
use tallyline_shared_kernel::api::Clock;

impl<R: ReportRepository, C: Clock> ReportingApplication<R, C> {
    /// Compares entered quantities using explicit bands without changing Drafts or approvals.
    ///
    /// # Errors
    /// Refuses an unavailable Draft, mismatched environment or repository failure.
    pub fn review_stock(
        &mut self,
        id: ReportId,
        snapshot: &ReferenceSnapshot,
        tolerance: ScuTolerance,
    ) -> Result<Vec<StockAssessment>, RepositoryError> {
        let report = self.draft_for_reference(id, snapshot)?;
        let now = self.clock.now();
        let staleness = snapshot.effective_staleness(None);
        Ok(report
            .rows()
            .iter()
            .filter_map(|row| {
                let value = row.stock()?;
                let prior = snapshot
                    .terminal_priors(report.key().terminal())
                    .and_then(|priors| {
                        priors.rows().iter().find(|prior| {
                            prior.commodity() == row.commodity()
                                && prior.side() == report.key().side()
                        })
                    });
                Some(StockAssessment::assess(
                    row.id(),
                    value,
                    prior,
                    tolerance,
                    now,
                    staleness,
                ))
            })
            .collect())
    }
}
