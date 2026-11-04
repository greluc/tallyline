// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Durable global submission blocks independent of the originating Report phase.

use crate::api::model::{
    JobId, JobPhase, NotProcessed, SubmissionHoldReason, SubmissionJob, SubmissionRepositoryError,
    SubmissionResult,
};

/// A service refusal that stops the complete submission queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionServiceCause {
    /// The shared app-token daily quota is exhausted.
    DailyQuota,
    /// The account or app-token recovery decision requires explicit acknowledgement.
    AccountPause,
    /// The configured write endpoint no longer exists.
    ApiChanged,
}
/// A durable service block bound to its exact committed originating answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubmissionServiceFence {
    cause: SubmissionServiceCause,
    source: JobId,
    source_revision: u64,
}
impl SubmissionServiceFence {
    /// Accepts only a positive committed source identity and revision.
    #[must_use]
    pub const fn new(
        cause: SubmissionServiceCause,
        source: JobId,
        source_revision: u64,
    ) -> Option<Self> {
        if source.0 == 0 || source_revision == 0 {
            return None;
        }
        Some(Self {
            cause,
            source,
            source_revision,
        })
    }
    /// Verifies the retained cause against its immutable originating job history.
    #[must_use]
    pub fn matches_source(self, job: &SubmissionJob) -> bool {
        if self.source != job.id() || job.service_pause_acknowledged() {
            return false;
        }
        let expected = if self.cause == SubmissionServiceCause::AccountPause
            && job
                .attempts()
                .last()
                .and_then(|attempt| attempt.account_evidence())
                .is_some()
        {
            job.attempts()
                .last()
                .and_then(|attempt| attempt.account_evidence())
                .and_then(|evidence| evidence.origin_revision())
        } else {
            match job.phase() {
                JobPhase::Held
                | JobPhase::Paused
                | JobPhase::PartiallyAccepted
                | JobPhase::OutcomeUnknown
                | JobPhase::Rejected => Some(job.revision()),
                JobPhase::Cancelled | JobPhase::Returned => job.revision().checked_sub(1),
                JobPhase::Queued | JobPhase::Sending | JobPhase::Succeeded => None,
            }
        };
        expected == Some(self.source_revision)
            && match self.cause {
                SubmissionServiceCause::AccountPause => job
                    .result()
                    .and_then(SubmissionResult::account_pause)
                    .is_some(),
                SubmissionServiceCause::DailyQuota => matches!(
                    job.result(),
                    Some(SubmissionResult::NotProcessed(NotProcessed::DailyQuota))
                ),
                SubmissionServiceCause::ApiChanged => matches!(
                    job.result(),
                    Some(SubmissionResult::Held(SubmissionHoldReason::ApiChanged))
                ),
            }
    }
    /// Returns the service-wide reason without credential or account data.
    #[must_use]
    pub const fn cause(self) -> SubmissionServiceCause {
        self.cause
    }
    /// Returns the immutable job identity of the originating answer.
    #[must_use]
    pub const fn source(self) -> JobId {
        self.source
    }
    /// Returns the exact committed revision that created the fence.
    #[must_use]
    pub const fn source_revision(self) -> u64 {
        self.source_revision
    }
}
/// Reads durable global service blocks through the submission persistence boundary.
pub trait SubmissionServiceFenceRepository {
    /// Loads every active service block; absence cannot replace a failed read.
    ///
    /// # Errors
    /// Refuses corrupt source identities or unavailable storage.
    fn service_fences(&mut self) -> Result<Vec<SubmissionServiceFence>, SubmissionRepositoryError>;
}
