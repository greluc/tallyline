// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Immutable submission jobs and persistence-only state.

mod storage;
mod submission_job;

pub use storage::{StoredSubmissionAttempt, StoredSubmissionJob};
pub use submission_job::{
    AttemptOutcome, JobId, JobPhase, NotProcessed, PayloadHash, QueueRefusal, SubmissionAttempt,
    SubmissionJob, SubmissionResult, SubmissionReturnReason,
};
