// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Save-before-publish attachment decisions.

use super::ReportingApplication;
use crate::api::model::{Report, ReportId};
use crate::api::{AttachmentEdit, ReportRepository, RepositoryError};
use tallyline_shared_kernel::api::Clock;

impl<R: ReportRepository, C: Clock> ReportingApplication<R, C> {
    /// Persists one value-bound attachment decision before publishing the changed Draft.
    ///
    /// # Errors
    /// Returns stale edits, invalid privacy/time decisions or unavailable storage.
    pub fn edit_attachment(
        &mut self,
        id: ReportId,
        expected: u64,
        edit: AttachmentEdit,
    ) -> Result<Report, RepositoryError> {
        let mut report = self
            .repository
            .load_draft(id)?
            .ok_or(RepositoryError::Invalid)?;
        if report.version() != expected {
            return Err(RepositoryError::Conflict);
        }
        report
            .edit_attachment(edit)
            .map_err(|_| RepositoryError::Invalid)?;
        if report.version() > expected {
            self.repository.save_draft(&report, expected)?;
        }
        Ok(report)
    }
}
