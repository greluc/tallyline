// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Exact-version explicit resolution commands without submission authority.

use super::SubmissionQueueController;
use crate::api::SubmissionControlError;
use tallyline_shared_kernel::api::model::SubmissionMode;
use tallyline_submission::api::model::{
    JobId, JobPhase, ReleaseToken, SubmissionCommand, SubmissionJob, UnknownDecision,
    UnknownResolutionAction,
};
use tallyline_submission::api::{CredentialApplication, SubmissionJobRepository};
use tallyline_workflows::api::UnknownDraft;

impl SubmissionQueueController {
    /// Records an explicit received or not-received statement for the displayed unknown origin.
    ///
    /// # Errors
    /// Refuses stale revisions, ineligible phases and failed local persistence.
    pub fn decide_unknown(
        &mut self,
        id: JobId,
        expected: u64,
        decision: UnknownDecision,
    ) -> Result<SubmissionJob, SubmissionControlError> {
        let job = self.unknown_job(id, expected)?;
        let change = job
            .apply(SubmissionCommand::ResolveUnknown(
                UnknownResolutionAction::Decision {
                    decision,
                    at: self.clock.now(),
                },
            ))
            .map_err(SubmissionControlError::Refused)?;
        SubmissionJobRepository::save(&mut self.store, &change)
            .map_err(SubmissionControlError::Repository)?;
        let committed = change.into_state();
        self.queue.update(committed.clone());
        Ok(committed)
    }

    /// Creates a separate unreviewed Draft after the displayed not-received decision.
    ///
    /// # Errors
    /// Refuses stale or ineligible origins and failed atomic persistence.
    pub fn duplicate_unknown(
        &mut self,
        id: JobId,
        expected: u64,
    ) -> Result<UnknownDraft, SubmissionControlError> {
        use tallyline_workflows::api::UnknownDraftStore as _;
        let duplicated = self
            .store
            .duplicate_unknown(id, expected, self.clock.now())
            .map_err(SubmissionControlError::Repository)?;
        self.queue.update(duplicated.origin.clone());
        Ok(duplicated)
    }

    fn unknown_job(
        &mut self,
        id: JobId,
        expected: u64,
    ) -> Result<SubmissionJob, SubmissionControlError> {
        let job = SubmissionJobRepository::load(&mut self.store, id)
            .map_err(SubmissionControlError::Repository)?
            .ok_or(SubmissionControlError::Invalid)?;
        if job.revision() != expected {
            return Err(SubmissionControlError::Conflict);
        }
        if job.phase() != JobPhase::OutcomeUnknown {
            return Err(SubmissionControlError::Invalid);
        }
        Ok(job)
    }
}

impl SubmissionQueueController {
    /// Starts a bounded read-only check for the exact displayed unknown origin.
    ///
    /// # Errors
    /// Refuses stale origins, missing current account validation and occupied read work.
    pub fn check_unknown(
        &mut self,
        id: JobId,
        expected: u64,
        credentials: &CredentialApplication,
        token: Option<&ReleaseToken>,
    ) -> Result<(), SubmissionControlError> {
        let job = self.unknown_job(id, expected)?;
        if job.request().header().mode != SubmissionMode::Production {
            return Err(SubmissionControlError::Invalid);
        }
        self.unknown.begin(&self.client, job, credentials, token)
    }
}
