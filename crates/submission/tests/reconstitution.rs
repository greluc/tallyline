// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Persistence reconstitution rejects ambiguous attempts instead of replaying them.

use tallyline_shared_kernel::api::model::{
    CommodityId, ReportId, RowId, SubmissionMode, TerminalId, Timestamp, TradeSide, UexEnvironment,
};
use tallyline_submission::api::model::{
    JobId, JobPhase, PayloadHash, StoredSubmissionAttempt, StoredSubmissionJob, SubmissionHeader,
    SubmissionJob, SubmissionRequest, SubmissionResult, SubmissionRow,
};

fn queued() -> Option<SubmissionJob> {
    SubmissionJob::queued(
        JobId(7),
        SubmissionRequest::new(
            SubmissionHeader {
                report: ReportId(4),
                report_version: 3,
                account: "test-account".to_owned(),
                mode: SubmissionMode::DryRun,
                terminal: TerminalId(89),
                side: TradeSide::Sell,
                environment: UexEnvironment::Live,
                game_version: "4.10.2".to_owned(),
                observed_at: Timestamp::from_seconds(100),
            },
            vec![
                SubmissionRow::new(
                    RowId(2),
                    CommodityId(7),
                    "120".parse().ok()?,
                    "0".parse().ok()?,
                    1,
                )
                .ok()?,
            ],
        )
        .ok()?,
    )
    .ok()
}
fn stored(job: &SubmissionJob) -> StoredSubmissionJob {
    StoredSubmissionJob {
        id: job.id(),
        revision: job.revision(),
        request: job.request().clone(),
        phase: job.phase(),
        result: job.result().cloned(),
        review_reason: job.review_reason(),
        attempts: job
            .attempts()
            .iter()
            .map(|attempt| StoredSubmissionAttempt {
                started_at: attempt.started_at(),
                rows: attempt.rows(),
                payload_hash: attempt.payload_hash(),
                date_added: attempt.date_added(),
                ended_at: attempt.ended_at(),
                outcome: attempt.outcome(),
            })
            .collect(),
    }
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_sub_9__reconstituted_jobs_preserve_fixed_snapshots_and_interrupted_sending_stays_closed() {
    let queued = queued().unwrap();
    let sending = queued
        .begin_prepared(
            Timestamp::from_seconds(110),
            PayloadHash::new([8; 32]),
            Timestamp::from_seconds(95),
        )
        .unwrap();
    let unknown = sending.recover(Timestamp::from_seconds(120)).unwrap();
    let success = sending
        .complete(
            SubmissionResult::DryRunSucceeded,
            Timestamp::from_seconds(120),
        )
        .unwrap();
    for job in [
        queued.clone(),
        queued.cancel().unwrap(),
        queued
            .return_for_review(
                tallyline_submission::api::model::SendReturnReason::GameVersionChanged,
            )
            .unwrap(),
        queued.recover(Timestamp::from_seconds(120)).unwrap(),
        sending.clone(),
        unknown.clone(),
        success,
    ] {
        assert_eq!(SubmissionJob::reconstitute(stored(&job)).unwrap(), job);
    }
    let restored = SubmissionJob::reconstitute(stored(&sending)).unwrap();
    assert_eq!(
        restored.recover(Timestamp::from_seconds(120)).unwrap(),
        unknown
    );
    assert!(restored.cancel().is_err());
    let snapshot = stored(&sending);
    let mut corrupt = snapshot.clone();
    corrupt.phase = JobPhase::Queued;
    assert!(SubmissionJob::reconstitute(corrupt).is_err());
    let mut corrupt = snapshot.clone();
    corrupt.revision = 1;
    assert!(SubmissionJob::reconstitute(corrupt).is_err());
    let mut corrupt = snapshot.clone();
    corrupt.attempts[0].rows = 2;
    assert!(SubmissionJob::reconstitute(corrupt).is_err());
    let mut corrupt = snapshot.clone();
    corrupt.attempts[0].date_added = Timestamp::from_seconds(101);
    assert!(SubmissionJob::reconstitute(corrupt).is_err());
    let mut corrupt = snapshot;
    corrupt.attempts[0].ended_at = Some(Timestamp::from_seconds(120));
    assert!(SubmissionJob::reconstitute(corrupt).is_err());
}
