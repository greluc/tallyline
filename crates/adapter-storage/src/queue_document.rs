// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Validated queue persistence without credentials or encoded request bodies.

use serde::{Deserialize, Serialize};
use tallyline_shared_kernel::api::model::{
    CommodityId, ReportId, RowId, SubmissionMode, TerminalId, Timestamp, TradeSide, UexEnvironment,
    UexReportId,
};
use tallyline_submission::api::model::SubmissionRepositoryError;
use tallyline_submission::api::model::{
    AttemptOutcome, JobId, JobPhase, NotProcessed, PayloadHash, SendReturnReason,
    StoredSubmissionAttempt, StoredSubmissionJob, SubmissionHeader, SubmissionJob,
    SubmissionRequest, SubmissionResult, SubmissionReturnReason, SubmissionRow, VerifiedUexReport,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JobDocument {
    header: HeaderDocument,
    rows: Vec<RowDocument>,
    newer_approvals: Vec<(u64, String)>,
    phase: PhaseDocument,
    attempts: Vec<AttemptDocument>,
    result: Option<ResultDocument>,
    review_reason: Option<ReturnDocument>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HeaderDocument {
    report: u64,
    report_version: u64,
    account: String,
    mode: ModeDocument,
    terminal: u32,
    side: SideDocument,
    environment: EnvironmentDocument,
    game_version: String,
    observed_at: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RowDocument {
    id: u64,
    commodity: u32,
    price: String,
    stock: String,
    status: u8,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttemptDocument {
    started_at: String,
    rows: usize,
    payload_hash: [u8; 32],
    date_added: String,
    ended_at: Option<String>,
    outcome: Option<OutcomeDocument>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResultDocument {
    ProductionAccepted(Vec<IdDocument>),
    ProductionSucceeded(Vec<ReceiptDocument>),
    DryRunSucceeded,
    Partial(Vec<IdDocument>),
    ConnectFailure,
    ScreenshotRequired,
    InvalidObservationTime,
    PreflightChanged,
    Unknown,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdDocument {
    id: u64,
    date_added: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptDocument {
    id: u64,
    row: u64,
    date_added: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OutcomeDocument {
    ProductionSucceeded,
    Unknown,
    DryRunSucceeded,
    Partial,
    ConnectFailure,
    ScreenshotRequired,
    InvalidObservationTime,
    PreflightChanged,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PhaseDocument {
    Queued,
    Succeeded,
    PartiallyAccepted,
    Sending,
    OutcomeUnknown,
    Returned,
    Cancelled,
    Held,
}
impl From<JobPhase> for PhaseDocument {
    fn from(value: JobPhase) -> Self {
        match value {
            JobPhase::Queued => Self::Queued,
            JobPhase::Succeeded => Self::Succeeded,
            JobPhase::PartiallyAccepted => Self::PartiallyAccepted,
            JobPhase::Sending => Self::Sending,
            JobPhase::OutcomeUnknown => Self::OutcomeUnknown,
            JobPhase::Returned => Self::Returned,
            JobPhase::Cancelled => Self::Cancelled,
            JobPhase::Held => Self::Held,
        }
    }
}
impl From<PhaseDocument> for JobPhase {
    fn from(value: PhaseDocument) -> Self {
        match value {
            PhaseDocument::Queued => Self::Queued,
            PhaseDocument::Succeeded => Self::Succeeded,
            PhaseDocument::PartiallyAccepted => Self::PartiallyAccepted,
            PhaseDocument::Sending => Self::Sending,
            PhaseDocument::OutcomeUnknown => Self::OutcomeUnknown,
            PhaseDocument::Returned => Self::Returned,
            PhaseDocument::Cancelled => Self::Cancelled,
            PhaseDocument::Held => Self::Held,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModeDocument {
    DryRun,
    Production,
}
impl From<SubmissionMode> for ModeDocument {
    fn from(value: SubmissionMode) -> Self {
        match value {
            SubmissionMode::DryRun => Self::DryRun,
            SubmissionMode::Production => Self::Production,
        }
    }
}
impl From<ModeDocument> for SubmissionMode {
    fn from(value: ModeDocument) -> Self {
        match value {
            ModeDocument::DryRun => Self::DryRun,
            ModeDocument::Production => Self::Production,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SideDocument {
    Buy,
    Sell,
}
impl From<TradeSide> for SideDocument {
    fn from(value: TradeSide) -> Self {
        match value {
            TradeSide::Buy => Self::Buy,
            TradeSide::Sell => Self::Sell,
        }
    }
}
impl From<SideDocument> for TradeSide {
    fn from(value: SideDocument) -> Self {
        match value {
            SideDocument::Buy => Self::Buy,
            SideDocument::Sell => Self::Sell,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EnvironmentDocument {
    Live,
    Ptu,
}
impl From<UexEnvironment> for EnvironmentDocument {
    fn from(value: UexEnvironment) -> Self {
        match value {
            UexEnvironment::Live => Self::Live,
            UexEnvironment::Ptu => Self::Ptu,
        }
    }
}
impl From<EnvironmentDocument> for UexEnvironment {
    fn from(value: EnvironmentDocument) -> Self {
        match value {
            EnvironmentDocument::Live => Self::Live,
            EnvironmentDocument::Ptu => Self::Ptu,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReturnDocument {
    ReviewRequired,
    GameVersionChanged,
    ObservationAgeUnconfirmed,
    ObservationTooOld,
    ObservationInFuture,
}
impl From<SendReturnReason> for ReturnDocument {
    fn from(value: SendReturnReason) -> Self {
        match value {
            SendReturnReason::ReviewRequired => Self::ReviewRequired,
            SendReturnReason::GameVersionChanged => Self::GameVersionChanged,
            SendReturnReason::ObservationAgeUnconfirmed => Self::ObservationAgeUnconfirmed,
            SendReturnReason::ObservationTooOld => Self::ObservationTooOld,
            SendReturnReason::ObservationInFuture => Self::ObservationInFuture,
        }
    }
}
impl From<ReturnDocument> for SendReturnReason {
    fn from(value: ReturnDocument) -> Self {
        match value {
            ReturnDocument::ReviewRequired => Self::ReviewRequired,
            ReturnDocument::GameVersionChanged => Self::GameVersionChanged,
            ReturnDocument::ObservationAgeUnconfirmed => Self::ObservationAgeUnconfirmed,
            ReturnDocument::ObservationTooOld => Self::ObservationTooOld,
            ReturnDocument::ObservationInFuture => Self::ObservationInFuture,
        }
    }
}

fn invalid<T>(_: T) -> SubmissionRepositoryError {
    SubmissionRepositoryError::Invalid
}
fn timestamp(value: &str) -> Result<Timestamp, SubmissionRepositoryError> {
    crate::reference_cache::timestamp(value).map_err(invalid)
}
fn id_document(value: &UexReportId) -> IdDocument {
    IdDocument {
        id: value.id(),
        date_added: value.date_added().millis().to_string(),
    }
}
fn restore_id(value: IdDocument) -> Result<UexReportId, SubmissionRepositoryError> {
    UexReportId::new(value.id, timestamp(&value.date_added)?)
        .ok_or(SubmissionRepositoryError::Invalid)
}
impl From<&SubmissionResult> for ResultDocument {
    fn from(value: &SubmissionResult) -> Self {
        match value {
            SubmissionResult::ProductionAccepted(ids) => {
                Self::ProductionAccepted(ids.iter().map(id_document).collect())
            }
            SubmissionResult::ProductionSucceeded(receipts) => Self::ProductionSucceeded(
                receipts
                    .iter()
                    .map(|value| ReceiptDocument {
                        id: value.id(),
                        row: value.row().0,
                        date_added: value.date_added().millis().to_string(),
                    })
                    .collect(),
            ),
            SubmissionResult::DryRunSucceeded => Self::DryRunSucceeded,
            SubmissionResult::Partial(ids) => Self::Partial(ids.iter().map(id_document).collect()),
            SubmissionResult::NotProcessed(NotProcessed::ConnectFailure) => Self::ConnectFailure,
            SubmissionResult::Returned(SubmissionReturnReason::ScreenshotRequired) => {
                Self::ScreenshotRequired
            }
            SubmissionResult::Returned(SubmissionReturnReason::InvalidObservationTime) => {
                Self::InvalidObservationTime
            }
            SubmissionResult::Returned(SubmissionReturnReason::PreflightChanged) => {
                Self::PreflightChanged
            }
            SubmissionResult::Unknown => Self::Unknown,
        }
    }
}
impl ResultDocument {
    fn restore(
        self,
        request: &SubmissionRequest,
        sent_at: Option<Timestamp>,
    ) -> Result<SubmissionResult, SubmissionRepositoryError> {
        Ok(match self {
            Self::ProductionAccepted(ids) => SubmissionResult::ProductionAccepted(
                ids.into_iter().map(restore_id).collect::<Result<_, _>>()?,
            ),
            Self::ProductionSucceeded(receipts) => SubmissionResult::ProductionSucceeded(
                receipts
                    .into_iter()
                    .map(|value| {
                        let row = request
                            .rows()
                            .iter()
                            .find(|row| row.id().0 == value.row)
                            .ok_or(SubmissionRepositoryError::Invalid)?;
                        VerifiedUexReport::from_lookup_at(
                            value.id,
                            request.header().terminal,
                            row.commodity(),
                            timestamp(&value.date_added)?,
                            request,
                            sent_at.ok_or(SubmissionRepositoryError::Invalid)?,
                        )
                        .map_err(invalid)
                    })
                    .collect::<Result<_, _>>()?,
            ),
            Self::DryRunSucceeded => SubmissionResult::DryRunSucceeded,
            Self::Partial(ids) => SubmissionResult::Partial(
                ids.into_iter().map(restore_id).collect::<Result<_, _>>()?,
            ),
            Self::ConnectFailure => SubmissionResult::NotProcessed(NotProcessed::ConnectFailure),
            Self::ScreenshotRequired => {
                SubmissionResult::Returned(SubmissionReturnReason::ScreenshotRequired)
            }
            Self::InvalidObservationTime => {
                SubmissionResult::Returned(SubmissionReturnReason::InvalidObservationTime)
            }
            Self::PreflightChanged => {
                SubmissionResult::Returned(SubmissionReturnReason::PreflightChanged)
            }
            Self::Unknown => SubmissionResult::Unknown,
        })
    }
}
impl From<AttemptOutcome> for OutcomeDocument {
    fn from(value: AttemptOutcome) -> Self {
        match value {
            AttemptOutcome::ProductionSucceeded => Self::ProductionSucceeded,
            AttemptOutcome::Returned(SubmissionReturnReason::PreflightChanged) => {
                Self::PreflightChanged
            }
            AttemptOutcome::Unknown => Self::Unknown,
            AttemptOutcome::DryRunSucceeded => Self::DryRunSucceeded,
            AttemptOutcome::Partial => Self::Partial,
            AttemptOutcome::NotProcessed(NotProcessed::ConnectFailure) => Self::ConnectFailure,
            AttemptOutcome::Returned(SubmissionReturnReason::ScreenshotRequired) => {
                Self::ScreenshotRequired
            }
            AttemptOutcome::Returned(SubmissionReturnReason::InvalidObservationTime) => {
                Self::InvalidObservationTime
            }
        }
    }
}
impl From<OutcomeDocument> for AttemptOutcome {
    fn from(value: OutcomeDocument) -> Self {
        match value {
            OutcomeDocument::ProductionSucceeded => Self::ProductionSucceeded,
            OutcomeDocument::PreflightChanged => {
                Self::Returned(SubmissionReturnReason::PreflightChanged)
            }
            OutcomeDocument::Unknown => Self::Unknown,
            OutcomeDocument::DryRunSucceeded => Self::DryRunSucceeded,
            OutcomeDocument::Partial => Self::Partial,
            OutcomeDocument::ConnectFailure => Self::NotProcessed(NotProcessed::ConnectFailure),
            OutcomeDocument::ScreenshotRequired => {
                Self::Returned(SubmissionReturnReason::ScreenshotRequired)
            }
            OutcomeDocument::InvalidObservationTime => {
                Self::Returned(SubmissionReturnReason::InvalidObservationTime)
            }
        }
    }
}
pub(crate) fn encode(job: &SubmissionJob) -> Result<String, SubmissionRepositoryError> {
    let request = job.request();
    let header = request.header();
    let document = JobDocument {
        header: HeaderDocument {
            report: header.report.0,
            report_version: header.report_version,
            account: header.account.clone(),
            mode: header.mode.into(),
            terminal: header.terminal.0,
            side: header.side.into(),
            environment: header.environment.into(),
            game_version: header.game_version.clone(),
            observed_at: header.observed_at.millis().to_string(),
        },
        rows: request
            .rows()
            .iter()
            .map(|row| RowDocument {
                id: row.id().0,
                commodity: row.commodity().0,
                price: row.price().to_string(),
                stock: row.stock().to_string(),
                status: row.status(),
            })
            .collect(),
        newer_approvals: request
            .newer_observation_confirmations()
            .iter()
            .map(|approval| {
                (
                    approval.row().0,
                    approval.observed_at().millis().to_string(),
                )
            })
            .collect(),
        phase: job.phase().into(),
        attempts: job
            .attempts()
            .iter()
            .map(|attempt| AttemptDocument {
                started_at: attempt.started_at().millis().to_string(),
                rows: attempt.rows(),
                payload_hash: attempt.payload_hash().bytes(),
                date_added: attempt.date_added().millis().to_string(),
                ended_at: attempt.ended_at().map(|at| at.millis().to_string()),
                outcome: attempt.outcome().map(Into::into),
            })
            .collect(),
        result: job.result().map(Into::into),
        review_reason: job.review_reason().map(Into::into),
    };
    serde_json::to_string(&document).map_err(invalid)
}
pub(crate) fn decode(
    id: JobId,
    revision: u64,
    payload: &str,
) -> Result<SubmissionJob, SubmissionRepositoryError> {
    let document: JobDocument = serde_json::from_str(payload).map_err(invalid)?;
    let header = document.header;
    let rows = document
        .rows
        .into_iter()
        .map(|row| {
            SubmissionRow::new(
                RowId(row.id),
                CommodityId(row.commodity),
                row.price.parse().map_err(invalid)?,
                row.stock.parse().map_err(invalid)?,
                row.status,
            )
            .map_err(invalid)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut request = SubmissionRequest::new(
        SubmissionHeader {
            report: ReportId(header.report),
            report_version: header.report_version,
            account: header.account,
            mode: header.mode.into(),
            terminal: TerminalId(header.terminal),
            side: header.side.into(),
            environment: header.environment.into(),
            game_version: header.game_version,
            observed_at: timestamp(&header.observed_at)?,
        },
        rows,
    )
    .map_err(invalid)?;
    for (row, observed) in document.newer_approvals {
        request = request
            .with_confirmed_newer_observation(RowId(row), timestamp(&observed)?)
            .map_err(invalid)?;
    }
    let attempts = document
        .attempts
        .into_iter()
        .map(|attempt| {
            Ok(StoredSubmissionAttempt {
                started_at: timestamp(&attempt.started_at)?,
                rows: attempt.rows,
                payload_hash: PayloadHash::new(attempt.payload_hash),
                date_added: timestamp(&attempt.date_added)?,
                ended_at: attempt.ended_at.map(|at| timestamp(&at)).transpose()?,
                outcome: attempt.outcome.map(Into::into),
            })
        })
        .collect::<Result<Vec<_>, SubmissionRepositoryError>>()?;
    let result = document
        .result
        .map(|result| result.restore(&request, attempts.last().map(|attempt| attempt.date_added)))
        .transpose()?;
    SubmissionJob::reconstitute(StoredSubmissionJob {
        id,
        revision,
        request,
        phase: document.phase.into(),
        attempts,
        result,
        review_reason: document.review_reason.map(Into::into),
    })
    .map_err(invalid)
}
pub(crate) const fn phase_name(phase: JobPhase) -> &'static str {
    match phase {
        JobPhase::Queued => "queued",
        JobPhase::Succeeded => "succeeded",
        JobPhase::PartiallyAccepted => "partially_accepted",
        JobPhase::Sending => "sending",
        JobPhase::OutcomeUnknown => "outcome_unknown",
        JobPhase::Returned => "returned",
        JobPhase::Cancelled => "cancelled",
        JobPhase::Held => "held",
    }
}

pub(crate) fn encode_mapping(
    mapping: &tallyline_submission::api::model::VerifiedReceiptMapping,
) -> Result<String, SubmissionRepositoryError> {
    let document: Vec<_> = mapping
        .receipts()
        .iter()
        .map(|receipt| ReceiptDocument {
            id: receipt.id(),
            row: receipt.row().0,
            date_added: receipt.date_added().millis().to_string(),
        })
        .collect();
    serde_json::to_string(&document).map_err(invalid)
}
pub(crate) fn decode_mapping(
    job: &SubmissionJob,
    payload: &str,
) -> Result<tallyline_submission::api::model::VerifiedReceiptMapping, SubmissionRepositoryError> {
    let documents: Vec<ReceiptDocument> = serde_json::from_str(payload).map_err(invalid)?;
    let restored = ResultDocument::ProductionSucceeded(documents).restore(
        job.request(),
        job.attempts().last().map(|attempt| attempt.date_added()),
    )?;
    let SubmissionResult::ProductionSucceeded(receipts) = restored else {
        return Err(SubmissionRepositoryError::Invalid);
    };
    tallyline_submission::api::model::VerifiedReceiptMapping::new(job, receipts).map_err(invalid)
}
