// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Submission outcomes applied through Reporting without a context dependency reversal.

use tallyline_reporting::api::model::{
    InvalidReport, Report, ReportChange, ReportDelivery, ReportFinding,
};
use tallyline_submission::api::model::{
    JobPhase, SendReturnReason, SubmissionJob, SubmissionResult, SubmissionReturnReason,
};

/// Translates a committed job outcome for its exact owning Report and release binding.
///
/// # Errors
/// Refuses another release identity or a conflicting terminal Report outcome.
pub fn apply_submission_delivery(
    report: &Report,
    job: &SubmissionJob,
) -> Result<Option<ReportChange>, InvalidReport> {
    let header = job.request().header();
    let binding = report.release_binding().ok_or(InvalidReport)?;
    if header.report != report.id()
        || binding.job() != Some(job.id())
        || header.account != binding.account()
        || header.mode != binding.mode()
        || header.environment != binding.environment()
    {
        return Err(InvalidReport);
    }
    let delivery = match job.phase() {
        JobPhase::Queued | JobPhase::Sending | JobPhase::Held => None,
        JobPhase::Succeeded => Some(ReportDelivery::FullAcceptance),
        JobPhase::PartiallyAccepted => Some(ReportDelivery::PartialAcceptance),
        JobPhase::OutcomeUnknown => Some(ReportDelivery::Unknown),
        JobPhase::Cancelled => Some(ReportDelivery::Return(ReportFinding::ReviewRequired)),
        JobPhase::Returned => {
            let finding = match (job.review_reason(), job.result()) {
                (Some(SendReturnReason::ReviewRequired), _) => ReportFinding::ReviewRequired,
                (Some(SendReturnReason::GameVersionChanged), _) => {
                    ReportFinding::GameVersionChanged
                }
                (Some(SendReturnReason::ObservationAgeUnconfirmed), _) => {
                    ReportFinding::ObservationAgeUnconfirmed
                }
                (Some(SendReturnReason::ObservationTooOld), _) => ReportFinding::ObservationTooOld,
                (Some(SendReturnReason::ObservationInFuture), _) => {
                    ReportFinding::ObservationInFuture
                }
                (
                    None,
                    Some(SubmissionResult::Returned(SubmissionReturnReason::ScreenshotRequired)),
                ) => ReportFinding::ScreenshotRequired,
                (
                    None,
                    Some(SubmissionResult::Returned(
                        SubmissionReturnReason::InvalidObservationTime,
                    )),
                ) => ReportFinding::InvalidObservationTime,
                (
                    None,
                    Some(SubmissionResult::Returned(SubmissionReturnReason::PreflightChanged)),
                ) => ReportFinding::ReviewRequired,
                (None, _) => ReportFinding::AppRestart,
            };
            Some(ReportDelivery::Return(finding))
        }
    };
    delivery
        .map(|delivery| report.apply_delivery(job.id(), delivery))
        .transpose()
}
