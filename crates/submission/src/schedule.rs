// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whole-report cooldown and observation-order checks over durable jobs.

use crate::api::model::{JobPhase, SubmissionJob, SubmissionResult};
use std::time::Duration;
use tallyline_shared_kernel::api::model::Timestamp;

const COOLDOWN: Duration = Duration::from_secs(5 * 60);

/// A closed scheduling decision without trimming any released row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleDecision {
    /// Every row can proceed to the remaining fresh release checks.
    Ready,
    /// Another durable attempt is already in flight, irrespective of account or mode.
    InFlight,
    /// At least one row still has a cooldown; the complete Report waits until this time.
    WaitingUntil(Timestamp),
    /// An older queued observation of a shared row key must be considered first.
    EarlierObservationPending,
    /// A newer observation of a shared row key was already accepted.
    NewerObservationSent,
    /// A newer possibly processed observation still needs explicit resolution.
    UnresolvedNewerObservation,
    /// The candidate phase or durable attempt metadata cannot authorise dispatch.
    Unavailable,
}

/// Decides scheduling from committed history without performing I/O or mutating a job.
pub struct SubmissionSchedule;
impl SubmissionSchedule {
    /// Checks global in-flight work, row cooldowns and oldest-observation ordering.
    ///
    /// The persistence port must supply all unfinished jobs and relevant retained history;
    /// unavailable history must fail at that port rather than become an empty history.
    #[must_use]
    pub fn assess(
        candidate: &SubmissionJob,
        history: &[SubmissionJob],
        now: Timestamp,
    ) -> ScheduleDecision {
        if candidate.phase() != JobPhase::Queued {
            return ScheduleDecision::Unavailable;
        }
        if history
            .iter()
            .any(|job| job.id() != candidate.id() && job.phase() == JobPhase::Sending)
        {
            return ScheduleDecision::InFlight;
        }
        if let Some(attempt) = candidate.attempts().last()
            && attempt.outcome()
                == Some(crate::api::model::AttemptOutcome::Held(
                    crate::api::model::SubmissionHoldReason::Cooldown,
                ))
            && let Some(until) = attempt.ended_at().and_then(|at| at.checked_add(COOLDOWN))
            && until > now
        {
            return ScheduleDecision::WaitingUntil(until);
        }
        let request = candidate.request();
        let key = request.header();
        let mut waiting_until = None;
        let mut earlier = false;
        let mut unresolved_newer = false;
        for job in history.iter().filter(|job| job.id() != candidate.id()) {
            let phase = job.observation_order_phase();
            let previous = job.request();
            let old = previous.header();
            if old.account != key.account || old.mode != key.mode || old.terminal != key.terminal {
                continue;
            }
            let shared = previous.rows().iter().any(|old_row| {
                request
                    .rows()
                    .iter()
                    .any(|row| row.commodity() == old_row.commodity())
            });
            if !shared {
                continue;
            }
            if old.side == key.side {
                if old.observed_at > key.observed_at {
                    match phase {
                        JobPhase::Succeeded => {
                            let unconfirmed = request.rows().iter().any(|row| {
                                previous
                                    .rows()
                                    .iter()
                                    .any(|old_row| row.commodity() == old_row.commodity())
                                    && !request
                                        .newer_observation_confirmed(row.id(), old.observed_at)
                            });
                            if unconfirmed {
                                return ScheduleDecision::NewerObservationSent;
                            }
                        }
                        JobPhase::PartiallyAccepted | JobPhase::OutcomeUnknown => {
                            unresolved_newer = true
                        }
                        JobPhase::Queued
                        | JobPhase::Sending
                        | JobPhase::Returned
                        | JobPhase::Cancelled
                        | JobPhase::Held
                        | JobPhase::Paused
                        | JobPhase::Rejected => continue,
                    }
                }
                if matches!(phase, JobPhase::Queued | JobPhase::Held)
                    && (old.observed_at, job.id()) < (key.observed_at, candidate.id())
                {
                    earlier = true;
                }
            }
            if matches!(
                phase,
                JobPhase::Succeeded | JobPhase::PartiallyAccepted | JobPhase::OutcomeUnknown
            ) || matches!(
                job.result()
                    .map(crate::api::model::SubmissionResult::final_result),
                Some(
                    SubmissionResult::Held(crate::api::model::SubmissionHoldReason::Cooldown)
                        | SubmissionResult::DuplicateEvidence(_)
                )
            ) {
                let Some(attempt) = job.attempts().last() else {
                    return ScheduleDecision::Unavailable;
                };
                let Some(ended_at) = attempt.ended_at() else {
                    return ScheduleDecision::Unavailable;
                };
                let origin = match job.result() {
                    Some(
                        SubmissionResult::ProductionSucceeded(_)
                        | SubmissionResult::ProductionAccepted(_),
                    ) => attempt.date_added(),
                    Some(
                        SubmissionResult::AccountRefusal { .. }
                        | SubmissionResult::DryRunSucceeded
                        | SubmissionResult::Unknown
                        | SubmissionResult::Partial(_)
                        | SubmissionResult::NotProcessed(_)
                        | SubmissionResult::Returned(_)
                        | SubmissionResult::Paused(_)
                        | SubmissionResult::Held(_)
                        | SubmissionResult::Rejected(_)
                        | SubmissionResult::NeedsOutcomeCheck { .. }
                        | SubmissionResult::PartialError { .. }
                        | SubmissionResult::Rechecked { .. }
                        | SubmissionResult::RateLimitedCode(_)
                        | SubmissionResult::DuplicateEvidence(_)
                        | SubmissionResult::PausedCode { .. },
                    )
                    | Some(SubmissionResult::VersionLockedOut(_))
                    | None => ended_at,
                };
                let Some(until) = origin.checked_add(COOLDOWN) else {
                    return ScheduleDecision::Unavailable;
                };
                if until > now {
                    waiting_until =
                        Some(waiting_until.map_or(until, |current: Timestamp| current.max(until)));
                }
            }
        }
        if unresolved_newer {
            ScheduleDecision::UnresolvedNewerObservation
        } else if earlier {
            ScheduleDecision::EarlierObservationPending
        } else {
            waiting_until.map_or(ScheduleDecision::Ready, ScheduleDecision::WaitingUntil)
        }
    }
}
