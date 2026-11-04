// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Explicit new-Draft creation from a retained unknown origin.

use tallyline_reporting::api::model::Report;
use tallyline_shared_kernel::api::model::Timestamp;
use tallyline_submission::api::model::{JobId, SubmissionJob, SubmissionRepositoryError};

/// The atomically committed origin annotation and separate editable Draft.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownDraft {
    /// The retained uncertain origin with its explicit duplication link.
    pub origin: SubmissionJob,
    /// The separate new Draft requiring complete review and release.
    pub draft: Report,
}

/// Atomically creates a separate Draft only after an explicit not-received decision.
pub trait UnknownDraftStore {
    /// Copies the exact displayed origin and retains both histories atomically.
    ///
    /// # Errors
    /// Refuses stale, unreviewed, already duplicated or positively resolved origins.
    fn duplicate_unknown(
        &mut self,
        id: JobId,
        expected_revision: u64,
        at: Timestamp,
    ) -> Result<UnknownDraft, SubmissionRepositoryError>;
}
