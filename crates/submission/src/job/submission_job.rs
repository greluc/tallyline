// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Immutable submission phases and write-ahead attempts.

use super::storage::StoredSubmissionJob;
use crate::api::model::{
    InvalidSubmission, SendReturnReason, SubmissionRequest, VerifiedUexReport,
};
use tallyline_shared_kernel::api::model::Timestamp;

pub use tallyline_shared_kernel::api::model::SubmissionJobId as JobId;

/// SHA-256 of the exact prepared request bytes, without those bytes in diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PayloadHash([u8; 32]);
impl PayloadHash {
    /// Wraps an adapter-computed digest.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    /// Returns the digest for durable storage.
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

/// The durable phase of a single submission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobPhase {
    /// Released and awaiting dispatch.
    Queued,
    /// Every request row was accepted; a test request creates no UEX rows.
    Succeeded,
    /// Some rows may be stored and require individual resolution.
    PartiallyAccepted,
    /// An attempt was committed before network dispatch.
    Sending,
    /// Sending may have stored rows and must never be replayed automatically.
    OutcomeUnknown,
    /// The Report requires renewed release.
    Returned,
    /// The user cancelled before sending began.
    Cancelled,
    /// A known unprocessed attempt awaits an explicit scheduling decision.
    Held,
}

/// A command refused without changing a submission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueRefusal {
    /// User cancellation is forbidden after dispatch begins.
    AlreadySending,
    /// This command cannot apply to the current phase.
    InvalidTransition,
    /// The prepared time is later than the observation or is not a whole second.
    InvalidPreparedPayload,
}

/// The durable result of one started attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttemptOutcome {
    /// Every production row was accepted, with any unverified identities kept unmapped.
    ProductionSucceeded,
    /// Processing cannot be excluded.
    Unknown,
    /// UEX validated a test request without storing any row.
    DryRunSucceeded,
    /// Some rows may have been stored without a complete mapping.
    Partial,
    /// The adapter established that no row could have been processed.
    NotProcessed(NotProcessed),
    /// A closed report-fixable refusal requires renewed release.
    Returned(SubmissionReturnReason),
}

/// A write-ahead record retained independently of restart recovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmissionAttempt {
    started_at: Timestamp,
    rows: usize,
    payload_hash: PayloadHash,
    date_added: Timestamp,
    ended_at: Option<Timestamp>,
    outcome: Option<AttemptOutcome>,
}
impl SubmissionAttempt {
    /// Returns the committed attempt start.
    #[must_use]
    pub const fn started_at(&self) -> Timestamp {
        self.started_at
    }
    /// Returns the complete request row count.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }
    /// Returns the digest of the exact request bytes.
    #[must_use]
    pub const fn payload_hash(&self) -> PayloadHash {
        self.payload_hash
    }
    /// Returns the exact whole-second timestamp encoded in the attempted body.
    #[must_use]
    pub const fn date_added(&self) -> Timestamp {
        self.date_added
    }
    /// Returns the recorded attempt completion time.
    #[must_use]
    pub const fn ended_at(&self) -> Option<Timestamp> {
        self.ended_at
    }
    /// Returns the completion outcome, absent while sending.
    #[must_use]
    pub const fn outcome(&self) -> Option<AttemptOutcome> {
        self.outcome
    }
}

/// A credential-free immutable submission and its durable attempts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmissionJob {
    id: JobId,
    revision: u64,
    request: SubmissionRequest,
    phase: JobPhase,
    attempts: Vec<SubmissionAttempt>,
    result: Option<SubmissionResult>,
    review_reason: Option<SendReturnReason>,
}
impl SubmissionJob {
    /// Creates a queued job for a valid released snapshot.
    ///
    /// # Errors
    /// Refuses a missing job identity.
    pub fn queued(id: JobId, request: SubmissionRequest) -> Result<Self, InvalidSubmission> {
        if id.0 == 0 {
            return Err(InvalidSubmission);
        }
        Ok(Self {
            id,
            revision: 1,
            request,
            phase: JobPhase::Queued,
            attempts: Vec::new(),
            result: None,
            review_reason: None,
        })
    }
    /// Returns the stable local identity.
    #[must_use]
    pub const fn id(&self) -> JobId {
        self.id
    }
    /// Returns the optimistic persistence version of this immutable job.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    fn advance(&self) -> Result<Self, QueueRefusal> {
        let mut next = self.clone();
        next.revision = self
            .revision
            .checked_add(1)
            .ok_or(QueueRefusal::InvalidTransition)?;
        Ok(next)
    }
    /// Borrows the fixed release snapshot.
    #[must_use]
    pub const fn request(&self) -> &SubmissionRequest {
        &self.request
    }
    /// Returns the current durable phase.
    #[must_use]
    pub const fn phase(&self) -> JobPhase {
        self.phase
    }
    /// Borrows every write-ahead attempt.
    #[must_use]
    pub fn attempts(&self) -> &[SubmissionAttempt] {
        &self.attempts
    }
    /// Starts one attempt; the repository must commit it before network dispatch.
    ///
    /// # Errors
    /// Refuses every phase except queued.
    pub fn begin(&self, now: Timestamp, payload_hash: PayloadHash) -> Result<Self, QueueRefusal> {
        self.begin_prepared(
            now,
            payload_hash,
            Timestamp::from_seconds(self.request.header().observed_at.seconds()),
        )
    }
    /// Starts an attempt whose recorded timestamp exactly matches the prepared body.
    ///
    /// # Errors
    /// Refuses non-queued jobs, fractional seconds and a later time than the released observation.
    pub fn begin_prepared(
        &self,
        now: Timestamp,
        payload_hash: PayloadHash,
        date_added: Timestamp,
    ) -> Result<Self, QueueRefusal> {
        if self.phase != JobPhase::Queued {
            return Err(QueueRefusal::InvalidTransition);
        }
        if date_added > self.request.header().observed_at
            || !date_added.millis().is_multiple_of(1000)
        {
            return Err(QueueRefusal::InvalidPreparedPayload);
        }
        let mut next = self.advance()?;
        next.phase = JobPhase::Sending;
        next.attempts.push(SubmissionAttempt {
            started_at: now,
            rows: self.request.rows().len(),
            payload_hash,
            date_added,
            ended_at: None,
            outcome: None,
        });
        Ok(next)
    }
    /// Cancels only a queued request.
    ///
    /// # Errors
    /// Refuses sending and final phases.
    pub fn cancel(&self) -> Result<Self, QueueRefusal> {
        match self.phase {
            JobPhase::Sending => Err(QueueRefusal::AlreadySending),
            JobPhase::Queued | JobPhase::Held => {
                let mut next = self.advance()?;
                next.phase = JobPhase::Cancelled;
                Ok(next)
            }
            JobPhase::OutcomeUnknown
            | JobPhase::Returned
            | JobPhase::Cancelled
            | JobPhase::Succeeded
            | JobPhase::PartiallyAccepted => Err(QueueRefusal::InvalidTransition),
        }
    }
    /// Returns the durable preflight reason requiring renewed manual release.
    #[must_use]
    pub const fn review_reason(&self) -> Option<SendReturnReason> {
        self.review_reason
    }
    /// Closes an unsent job after a report-fixable preflight refusal.
    ///
    /// # Errors
    /// Refuses sending and final phases without changing attempts or the released snapshot.
    pub fn return_for_review(&self, reason: SendReturnReason) -> Result<Self, QueueRefusal> {
        match self.phase {
            JobPhase::Queued | JobPhase::Held => {
                let mut next = self.advance()?;
                next.phase = JobPhase::Returned;
                next.review_reason = Some(reason);
                Ok(next)
            }
            JobPhase::Sending => Err(QueueRefusal::AlreadySending),
            JobPhase::OutcomeUnknown
            | JobPhase::Returned
            | JobPhase::Cancelled
            | JobPhase::Succeeded
            | JobPhase::PartiallyAccepted => Err(QueueRefusal::InvalidTransition),
        }
    }
    /// Closes interrupted work without ever making it dispatchable again.
    ///
    /// # Errors
    /// Refuses corrupt sending state without a write-ahead attempt.
    pub fn recover(&self, now: Timestamp) -> Result<Self, QueueRefusal> {
        let mut next = self.clone();
        match self.phase {
            JobPhase::Queued | JobPhase::Held => {
                next = self.advance()?;
                next.phase = JobPhase::Returned;
            }
            JobPhase::Sending => {
                next = self.advance()?;
                next.phase = JobPhase::OutcomeUnknown;
                let attempt = next
                    .attempts
                    .last_mut()
                    .ok_or(QueueRefusal::InvalidTransition)?;
                attempt.outcome = Some(AttemptOutcome::Unknown);
                next.result = Some(SubmissionResult::Unknown);
                attempt.ended_at = Some(now);
            }
            JobPhase::OutcomeUnknown
            | JobPhase::Returned
            | JobPhase::Cancelled
            | JobPhase::Succeeded
            | JobPhase::PartiallyAccepted => return Ok(next),
        }
        Ok(next)
    }
}

/// A closed transport result without speculative UEX row identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmissionResult {
    /// Every positive remote identity was verified against its released row.
    ProductionSucceeded(Vec<VerifiedUexReport>),
    /// UEX accepted every row; positive returned identities remain unmapped until verified.
    ProductionAccepted(Vec<tallyline_shared_kernel::api::model::UexReportId>),
    /// Test validation succeeded and created no UEX reports.
    DryRunSucceeded,
    /// Rows may have been stored and need individual resolution.
    Partial(Vec<tallyline_shared_kernel::api::model::UexReportId>),
    /// The adapter established that no row could have been processed.
    NotProcessed(NotProcessed),
    /// A closed report-fixable refusal requires renewed release.
    Returned(SubmissionReturnReason),
    /// Processing cannot be excluded.
    Unknown,
}
impl SubmissionJob {
    /// Returns the completed result without inventing row identities.
    #[must_use]
    pub const fn result(&self) -> Option<&SubmissionResult> {
        self.result.as_ref()
    }
    /// Closes one sending attempt exactly once.
    ///
    /// # Errors
    /// Refuses a final job, invalid attempt state or dry-run success for production.
    pub fn complete(&self, result: SubmissionResult, now: Timestamp) -> Result<Self, QueueRefusal> {
        if self.phase != JobPhase::Sending {
            return Err(QueueRefusal::InvalidTransition);
        }
        if result == SubmissionResult::DryRunSucceeded
            && self.request.header().mode
                != tallyline_shared_kernel::api::model::SubmissionMode::DryRun
        {
            return Err(QueueRefusal::InvalidTransition);
        }
        if self.request.header().mode == tallyline_shared_kernel::api::model::SubmissionMode::DryRun
            && matches!(&result, SubmissionResult::Partial(ids) if !ids.is_empty())
        {
            return Err(QueueRefusal::InvalidTransition);
        }
        if let SubmissionResult::ProductionAccepted(ids) = &result {
            let attempt = self
                .attempts
                .last()
                .ok_or(QueueRefusal::InvalidTransition)?;
            let mut seen = std::collections::BTreeSet::new();
            if self.request.header().mode
                != tallyline_shared_kernel::api::model::SubmissionMode::Production
                || ids.len() != self.request.rows().len()
                || ids
                    .iter()
                    .any(|id| id.date_added() != attempt.date_added() || !seen.insert(id.id()))
            {
                return Err(QueueRefusal::InvalidTransition);
            }
        }
        if let SubmissionResult::ProductionSucceeded(receipts) = &result {
            let sent_at = self
                .attempts
                .last()
                .ok_or(QueueRefusal::InvalidTransition)?
                .date_added();
            let mut rows = std::collections::BTreeSet::new();
            let mut ids = std::collections::BTreeSet::new();
            if self.request.header().mode
                != tallyline_shared_kernel::api::model::SubmissionMode::Production
                || receipts.len() != self.request.rows().len()
                || receipts.iter().any(|receipt| {
                    !receipt.matches_at(&self.request, sent_at)
                        || !rows.insert(receipt.row().0)
                        || !ids.insert(receipt.id())
                })
            {
                return Err(QueueRefusal::InvalidTransition);
            }
        }
        let mut next = self.advance()?;
        let attempt = next
            .attempts
            .last_mut()
            .ok_or(QueueRefusal::InvalidTransition)?;
        let (phase, outcome) = match &result {
            SubmissionResult::DryRunSucceeded => {
                (JobPhase::Succeeded, AttemptOutcome::DryRunSucceeded)
            }
            SubmissionResult::ProductionSucceeded(_) | SubmissionResult::ProductionAccepted(_) => {
                (JobPhase::Succeeded, AttemptOutcome::ProductionSucceeded)
            }
            SubmissionResult::Partial(_) => (JobPhase::PartiallyAccepted, AttemptOutcome::Partial),
            SubmissionResult::Unknown => (JobPhase::OutcomeUnknown, AttemptOutcome::Unknown),
            SubmissionResult::NotProcessed(reason) => {
                (JobPhase::Held, AttemptOutcome::NotProcessed(*reason))
            }
            SubmissionResult::Returned(reason) => {
                (JobPhase::Returned, AttemptOutcome::Returned(*reason))
            }
        };
        next.phase = phase;
        next.result = Some(result);
        attempt.outcome = Some(outcome);
        attempt.ended_at = Some(now);
        Ok(next)
    }
}

/// Verified failures that precede storage of any Report row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotProcessed {
    /// Connection refusal or a connect-phase timeout before HTTP request writing.
    ConnectFailure,
}

/// A report-fixable outcome that requires a new release decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionReturnReason {
    /// The account must attach confirmed private screenshot evidence.
    ScreenshotRequired,
    /// The application proved that final checks failed before any HTTP request started.
    PreflightChanged,
    /// UEX refused the encoded observation time; review the time and system clock.
    InvalidObservationTime,
}

impl SubmissionJob {
    /// Reconstitutes validated durable state for the persistence adapter.
    ///
    /// # Errors
    /// Refuses missing, inconsistent or unsupported attempt history rather than inventing state.
    pub fn reconstitute(stored: StoredSubmissionJob) -> Result<Self, InvalidSubmission> {
        let mut job = Self::queued(stored.id, stored.request)?;
        match stored.attempts.as_slice() {
            [] => {
                if stored.result.is_some() {
                    return Err(InvalidSubmission);
                }
            }
            [attempt] => {
                if attempt.rows != job.request.rows().len() {
                    return Err(InvalidSubmission);
                }
                job = job
                    .begin_prepared(attempt.started_at, attempt.payload_hash, attempt.date_added)
                    .map_err(|_| InvalidSubmission)?;
                job = match (attempt.ended_at, attempt.outcome, stored.result.as_ref()) {
                    (Some(ended_at), Some(_), Some(result)) => job
                        .complete(result.clone(), ended_at)
                        .map_err(|_| InvalidSubmission)?,
                    (None, None, None) => job,
                    _ => return Err(InvalidSubmission),
                };
            }
            _ => return Err(InvalidSubmission),
        }
        if let Some(reason) = stored.review_reason {
            job = job
                .return_for_review(reason)
                .map_err(|_| InvalidSubmission)?;
        }
        if job.phase != stored.phase {
            job = match stored.phase {
                JobPhase::Cancelled => job.cancel(),
                JobPhase::Returned => job.recover(Timestamp::from_seconds(0)),
                JobPhase::Queued
                | JobPhase::Succeeded
                | JobPhase::PartiallyAccepted
                | JobPhase::Sending
                | JobPhase::OutcomeUnknown
                | JobPhase::Held => return Err(InvalidSubmission),
            }
            .map_err(|_| InvalidSubmission)?;
        }
        let attempts: Vec<_> = stored
            .attempts
            .into_iter()
            .map(|attempt| SubmissionAttempt {
                started_at: attempt.started_at,
                rows: attempt.rows,
                payload_hash: attempt.payload_hash,
                date_added: attempt.date_added,
                ended_at: attempt.ended_at,
                outcome: attempt.outcome,
            })
            .collect();
        if job.phase != stored.phase
            || job.revision != stored.revision
            || job.attempts != attempts
            || job.result != stored.result
            || job.review_reason != stored.review_reason
        {
            return Err(InvalidSubmission);
        }
        Ok(job)
    }
}
