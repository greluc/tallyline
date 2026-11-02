// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Atomic manual reassessment and immutable review publication.

use super::{
    Clock, ReferenceSnapshot, ReportId, ReportRepository, ReportingApplication, RepositoryError,
};
use crate::api::model::ManualReview;
impl<R: ReportRepository, C: Clock> ReportingApplication<R, C> {
    /// Reassesses prices, stocks and statuses atomically and returns the committed Draft with its assessments.
    ///
    /// # Errors
    /// Refuses stale versions, unavailable exact comparisons, mismatched references or storage failure.
    pub fn review_manual(
        &mut self,
        id: ReportId,
        expected: u64,
        snapshot: &ReferenceSnapshot,
    ) -> Result<ManualReview, RepositoryError> {
        let report = self.draft_for_reference(id, snapshot)?;
        if report.version() != expected {
            return Err(RepositoryError::Conflict);
        }
        let prices = self.price_assessments(&report, snapshot);
        if prices
            .iter()
            .any(|assessment| assessment.unavailable().is_some())
        {
            return Err(RepositoryError::AssessmentUnavailable);
        }
        let stocks = self.stock_assessments(&report, snapshot, self.stock_settings.tolerance());
        if stocks
            .iter()
            .any(|assessment| assessment.unavailable().is_some())
        {
            return Err(RepositoryError::AssessmentUnavailable);
        }
        let statuses = self.status_assessments(&report, snapshot);
        let next = report
            .reassess_prices(&prices, snapshot.environment())
            .and_then(|report| report.reassess_stocks(&stocks, snapshot.environment()))
            .and_then(|report| report.reassess_status(&statuses, snapshot.environment()))
            .map_err(|_| RepositoryError::Invalid)?;
        if next.version() != expected {
            self.repository.save_draft(&next, expected)?;
        }
        Ok(ManualReview::new(
            next,
            self.stock_settings,
            prices,
            stocks,
            statuses,
        ))
    }
}
