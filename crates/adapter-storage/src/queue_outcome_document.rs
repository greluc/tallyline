// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Durable closed-result and attempt-outcome documents without credential data.

use crate::queue_account_evidence::AccountCheckDocument;
use crate::queue_client_version::ClientVersionDocument;
use serde::{Deserialize, Serialize};
use tallyline_shared_kernel::api::model::{Timestamp, UexReportId};
use tallyline_submission::api::model::{
    AttemptOutcome, JobId, NotProcessed, SubmissionCorrection, SubmissionRejection,
    SubmissionRepositoryError, SubmissionRequest, SubmissionResult, SubmissionReturnReason,
    UexErrorCode, VerifiedUexReport,
};

fn invalid<T>(_: T) -> SubmissionRepositoryError {
    SubmissionRepositoryError::Invalid
}
fn timestamp(value: &str) -> Result<Timestamp, SubmissionRepositoryError> {
    crate::reference_cache::timestamp(value).map_err(invalid)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ResultDocument {
    AccountRefusal(
        Option<String>,
        u16,
        AccountCheckDocument,
        Box<ResultDocument>,
        Box<ResultDocument>,
    ),
    PausedCode(String, Box<ResultDocument>),
    DuplicateEvidence(u64),
    RateLimitedCode(String),
    Rechecked(String, Box<ResultDocument>),
    PartialError(String, Option<usize>),
    NeedsOutcomeCheck(String, u16),
    Rejected(String),
    PermanentRefusal(String, CorrectionDocument),
    ProductionAccepted(Vec<IdDocument>),
    ProductionSucceeded(Vec<ReceiptDocument>),
    DryRunSucceeded,
    Partial(Vec<IdDocument>),
    AppToken,
    UpdateRequired,
    VersionLockedOut(Option<ClientVersionDocument>),
    KeyChecked,
    Reauthenticate,
    KeyCheckUnavailable,
    AcceptanceClosed,
    DuplicateCooldown,
    ApiChanged,
    ConnectFailure,
    ServiceUnavailable,
    RateLimited,
    RowBudget,
    ImageUpload,
    ImageStorage,
    DailyQuota,

    ScreenshotRequired,
    InvalidObservationTime,
    PreflightChanged,
    PossiblyAlreadyReceived,
    MaxRowsExceeded,
    TerminalNotFound,
    InvalidScreenshot,
    ScreenshotTooLarge,

    Unknown,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdDocument {
    pub(crate) id: u64,
    pub(crate) date_added: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReceiptDocument {
    pub(crate) id: u64,
    pub(crate) row: u64,
    pub(crate) date_added: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OutcomeDocument {
    OutcomeCheckRequired,
    Rejected,
    ProductionSucceeded,
    Unknown,
    DryRunSucceeded,
    Partial,
    AppToken,
    UpdateRequired,
    VersionLockedOut(Option<ClientVersionDocument>),
    KeyChecked,
    Reauthenticate,
    KeyCheckUnavailable,
    AcceptanceClosed,
    DuplicateCooldown,
    ApiChanged,
    ConnectFailure,
    ServiceUnavailable,
    RateLimited,
    RowBudget,
    ImageUpload,
    ImageStorage,
    DailyQuota,

    ScreenshotRequired,
    InvalidObservationTime,
    PreflightChanged,
    PossiblyAlreadyReceived,
    MaxRowsExceeded,
    TerminalNotFound,
    InvalidScreenshot,
    ScreenshotTooLarge,
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
            SubmissionResult::NotProcessed(NotProcessed::ServiceUnavailable) => {
                Self::ServiceUnavailable
            }
            SubmissionResult::Returned(SubmissionReturnReason::ScreenshotRequired) => {
                Self::ScreenshotRequired
            }
            SubmissionResult::Returned(SubmissionReturnReason::InvalidObservationTime) => {
                Self::InvalidObservationTime
            }
            SubmissionResult::Returned(SubmissionReturnReason::PreflightChanged) => {
                Self::PreflightChanged
            }
            SubmissionResult::Returned(SubmissionReturnReason::MaxRowsExceeded) => {
                Self::MaxRowsExceeded
            }
            SubmissionResult::Returned(SubmissionReturnReason::TerminalNotFound) => {
                Self::TerminalNotFound
            }
            SubmissionResult::Returned(SubmissionReturnReason::InvalidScreenshot) => {
                Self::InvalidScreenshot
            }
            SubmissionResult::Returned(SubmissionReturnReason::ScreenshotTooLarge) => {
                Self::ScreenshotTooLarge
            }
            SubmissionResult::AccountRefusal {
                code,
                http_status,
                reason,
                check,
                outcome,
            } => Self::AccountRefusal(
                code.as_ref().map(|code| code.as_str().to_owned()),
                *http_status,
                (*check).into(),
                Box::new(Self::from(&SubmissionResult::Paused(*reason))),
                Box::new(Self::from(outcome.as_ref())),
            ),
            SubmissionResult::PartialError { code, row } => {
                Self::PartialError(code.as_str().to_owned(), *row)
            }
            SubmissionResult::NeedsOutcomeCheck { code, http_status } => {
                Self::NeedsOutcomeCheck(code.as_str().to_owned(), *http_status)
            }
            SubmissionResult::Rechecked { code, outcome } => Self::Rechecked(
                code.as_str().to_owned(),
                Box::new(Self::from(outcome.as_ref())),
            ),
            SubmissionResult::RateLimitedCode(code) => {
                Self::RateLimitedCode(code.as_str().to_owned())
            }
            SubmissionResult::NotProcessed(NotProcessed::RateLimited) => Self::RateLimited,
            SubmissionResult::NotProcessed(NotProcessed::RowBudget) => Self::RowBudget,
            SubmissionResult::NotProcessed(NotProcessed::ImageUpload) => Self::ImageUpload,
            SubmissionResult::NotProcessed(NotProcessed::ImageStorage) => Self::ImageStorage,
            SubmissionResult::NotProcessed(NotProcessed::DailyQuota) => Self::DailyQuota,
            SubmissionResult::Held(
                tallyline_submission::api::model::SubmissionHoldReason::Cooldown,
            ) => Self::DuplicateCooldown,
            SubmissionResult::DuplicateEvidence(job) => Self::DuplicateEvidence(job.0),
            SubmissionResult::Returned(SubmissionReturnReason::PossiblyAlreadyReceived) => {
                Self::PossiblyAlreadyReceived
            }
            SubmissionResult::PausedCode { code, reason } => Self::PausedCode(
                code.as_str().to_owned(),
                Box::new(Self::from(&SubmissionResult::Paused(*reason))),
            ),
            SubmissionResult::Unknown => Self::Unknown,
            SubmissionResult::Rejected(refusal) => Self::PermanentRefusal(
                refusal.code().as_str().to_owned(),
                refusal.correction().into(),
            ),
            SubmissionResult::VersionLockedOut(version) => {
                Self::VersionLockedOut(version.map(ClientVersionDocument))
            }
            SubmissionResult::Paused(reason) => match reason {
                tallyline_submission::api::model::SubmissionPauseReason::AppToken => Self::AppToken,
                tallyline_submission::api::model::SubmissionPauseReason::UpdateRequired => {
                    Self::UpdateRequired
                }
                tallyline_submission::api::model::SubmissionPauseReason::KeyChecked => {
                    Self::KeyChecked
                }
                tallyline_submission::api::model::SubmissionPauseReason::Reauthenticate => {
                    Self::Reauthenticate
                }
                tallyline_submission::api::model::SubmissionPauseReason::KeyCheckUnavailable => {
                    Self::KeyCheckUnavailable
                }
            },
            SubmissionResult::Held(
                tallyline_submission::api::model::SubmissionHoldReason::AcceptanceClosed,
            ) => Self::AcceptanceClosed,
            SubmissionResult::Held(
                tallyline_submission::api::model::SubmissionHoldReason::ApiChanged,
            ) => Self::ApiChanged,
        }
    }
}
impl ResultDocument {
    pub(crate) fn restore(
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
            Self::ServiceUnavailable => {
                SubmissionResult::NotProcessed(NotProcessed::ServiceUnavailable)
            }
            Self::ScreenshotRequired => {
                SubmissionResult::Returned(SubmissionReturnReason::ScreenshotRequired)
            }
            Self::InvalidObservationTime => {
                SubmissionResult::Returned(SubmissionReturnReason::InvalidObservationTime)
            }
            Self::PreflightChanged => {
                SubmissionResult::Returned(SubmissionReturnReason::PreflightChanged)
            }
            Self::VersionLockedOut(version) => {
                SubmissionResult::VersionLockedOut(version.map(|version| version.0))
            }
            Self::KeyChecked => SubmissionResult::Paused(
                tallyline_submission::api::model::SubmissionPauseReason::KeyChecked,
            ),
            Self::Reauthenticate => SubmissionResult::Paused(
                tallyline_submission::api::model::SubmissionPauseReason::Reauthenticate,
            ),
            Self::KeyCheckUnavailable => SubmissionResult::Paused(
                tallyline_submission::api::model::SubmissionPauseReason::KeyCheckUnavailable,
            ),
            Self::MaxRowsExceeded => {
                SubmissionResult::Returned(SubmissionReturnReason::MaxRowsExceeded)
            }
            Self::TerminalNotFound => {
                SubmissionResult::Returned(SubmissionReturnReason::TerminalNotFound)
            }
            Self::InvalidScreenshot => {
                SubmissionResult::Returned(SubmissionReturnReason::InvalidScreenshot)
            }
            Self::ScreenshotTooLarge => {
                SubmissionResult::Returned(SubmissionReturnReason::ScreenshotTooLarge)
            }
            Self::AccountRefusal(code, http_status, check, reason, outcome) => {
                let SubmissionResult::Paused(reason) = reason.restore(request, sent_at)? else {
                    return Err(SubmissionRepositoryError::Invalid);
                };
                SubmissionResult::AccountRefusal {
                    code: match code {
                        Some(code) => Some(
                            tallyline_submission::api::model::UexErrorCode::new(code)
                                .ok_or(SubmissionRepositoryError::Invalid)?,
                        ),
                        None => None,
                    },
                    http_status,
                    check: check.restore(request, sent_at)?,
                    reason,
                    outcome: Box::new(outcome.restore(request, sent_at)?),
                }
            }
            Self::PartialError(code, row) => SubmissionResult::PartialError {
                code: tallyline_submission::api::model::UexErrorCode::new(code)
                    .ok_or(SubmissionRepositoryError::Invalid)?,
                row,
            },
            Self::NeedsOutcomeCheck(code, http_status) => SubmissionResult::NeedsOutcomeCheck {
                code: tallyline_submission::api::model::UexErrorCode::new(code)
                    .ok_or(SubmissionRepositoryError::Invalid)?,
                http_status,
            },
            Self::Rechecked(code, outcome) => SubmissionResult::Rechecked {
                code: tallyline_submission::api::model::UexErrorCode::new(code)
                    .ok_or(SubmissionRepositoryError::Invalid)?,
                outcome: Box::new(outcome.restore(request, sent_at)?),
            },
            Self::RateLimitedCode(code) => SubmissionResult::RateLimitedCode(
                tallyline_submission::api::model::UexErrorCode::new(code)
                    .ok_or(SubmissionRepositoryError::Invalid)?,
            ),
            Self::RateLimited => SubmissionResult::NotProcessed(NotProcessed::RateLimited),
            Self::RowBudget => SubmissionResult::NotProcessed(NotProcessed::RowBudget),
            Self::ImageUpload => SubmissionResult::NotProcessed(NotProcessed::ImageUpload),
            Self::ImageStorage => SubmissionResult::NotProcessed(NotProcessed::ImageStorage),
            Self::DailyQuota => SubmissionResult::NotProcessed(NotProcessed::DailyQuota),
            Self::UpdateRequired => SubmissionResult::Paused(
                tallyline_submission::api::model::SubmissionPauseReason::UpdateRequired,
            ),
            Self::DuplicateCooldown => SubmissionResult::Held(
                tallyline_submission::api::model::SubmissionHoldReason::Cooldown,
            ),
            Self::DuplicateEvidence(id) => SubmissionResult::DuplicateEvidence(JobId(id)),
            Self::PossiblyAlreadyReceived => {
                SubmissionResult::Returned(SubmissionReturnReason::PossiblyAlreadyReceived)
            }
            Self::PausedCode(code, outcome) => {
                let SubmissionResult::Paused(reason) = outcome.restore(request, sent_at)? else {
                    return Err(SubmissionRepositoryError::Invalid);
                };
                SubmissionResult::PausedCode {
                    code: tallyline_submission::api::model::UexErrorCode::new(code)
                        .ok_or(SubmissionRepositoryError::Invalid)?,
                    reason,
                }
            }
            Self::Unknown => SubmissionResult::Unknown,
            Self::Rejected(code) => {
                let correction = if code == "invalid_game_version" {
                    SubmissionCorrection::NewCapture
                } else {
                    SubmissionCorrection::ReviewClient
                };
                SubmissionResult::Rejected(SubmissionRejection::new(
                    UexErrorCode::new(code).ok_or(SubmissionRepositoryError::Invalid)?,
                    correction,
                ))
            }
            Self::PermanentRefusal(code, correction) => {
                SubmissionResult::Rejected(SubmissionRejection::new(
                    UexErrorCode::new(code).ok_or(SubmissionRepositoryError::Invalid)?,
                    correction.into(),
                ))
            }
            Self::AppToken => SubmissionResult::Paused(
                tallyline_submission::api::model::SubmissionPauseReason::AppToken,
            ),
            Self::AcceptanceClosed => SubmissionResult::Held(
                tallyline_submission::api::model::SubmissionHoldReason::AcceptanceClosed,
            ),
            Self::ApiChanged => SubmissionResult::Held(
                tallyline_submission::api::model::SubmissionHoldReason::ApiChanged,
            ),
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
            AttemptOutcome::Returned(SubmissionReturnReason::MaxRowsExceeded) => {
                Self::MaxRowsExceeded
            }
            AttemptOutcome::Returned(SubmissionReturnReason::TerminalNotFound) => {
                Self::TerminalNotFound
            }
            AttemptOutcome::Returned(SubmissionReturnReason::InvalidScreenshot) => {
                Self::InvalidScreenshot
            }
            AttemptOutcome::Returned(SubmissionReturnReason::ScreenshotTooLarge) => {
                Self::ScreenshotTooLarge
            }
            AttemptOutcome::OutcomeCheckRequired => Self::OutcomeCheckRequired,
            AttemptOutcome::NotProcessed(NotProcessed::RateLimited) => Self::RateLimited,
            AttemptOutcome::NotProcessed(NotProcessed::RowBudget) => Self::RowBudget,
            AttemptOutcome::NotProcessed(NotProcessed::ImageUpload) => Self::ImageUpload,
            AttemptOutcome::NotProcessed(NotProcessed::ImageStorage) => Self::ImageStorage,
            AttemptOutcome::NotProcessed(NotProcessed::DailyQuota) => Self::DailyQuota,
            AttemptOutcome::Held(
                tallyline_submission::api::model::SubmissionHoldReason::Cooldown,
            ) => Self::DuplicateCooldown,
            AttemptOutcome::Returned(SubmissionReturnReason::PossiblyAlreadyReceived) => {
                Self::PossiblyAlreadyReceived
            }
            AttemptOutcome::Unknown => Self::Unknown,
            AttemptOutcome::Rejected => Self::Rejected,
            AttemptOutcome::VersionLockedOut(version) => {
                Self::VersionLockedOut(version.map(ClientVersionDocument))
            }
            AttemptOutcome::Paused(reason) => match reason {
                tallyline_submission::api::model::SubmissionPauseReason::AppToken => Self::AppToken,
                tallyline_submission::api::model::SubmissionPauseReason::UpdateRequired => {
                    Self::UpdateRequired
                }
                tallyline_submission::api::model::SubmissionPauseReason::KeyChecked => {
                    Self::KeyChecked
                }
                tallyline_submission::api::model::SubmissionPauseReason::Reauthenticate => {
                    Self::Reauthenticate
                }
                tallyline_submission::api::model::SubmissionPauseReason::KeyCheckUnavailable => {
                    Self::KeyCheckUnavailable
                }
            },
            AttemptOutcome::Held(
                tallyline_submission::api::model::SubmissionHoldReason::AcceptanceClosed,
            ) => Self::AcceptanceClosed,
            AttemptOutcome::Held(
                tallyline_submission::api::model::SubmissionHoldReason::ApiChanged,
            ) => Self::ApiChanged,
            AttemptOutcome::DryRunSucceeded => Self::DryRunSucceeded,
            AttemptOutcome::Partial => Self::Partial,
            AttemptOutcome::NotProcessed(NotProcessed::ConnectFailure) => Self::ConnectFailure,
            AttemptOutcome::NotProcessed(NotProcessed::ServiceUnavailable) => {
                Self::ServiceUnavailable
            }
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
            OutcomeDocument::VersionLockedOut(version) => {
                Self::VersionLockedOut(version.map(|version| version.0))
            }
            OutcomeDocument::ProductionSucceeded => Self::ProductionSucceeded,
            OutcomeDocument::PreflightChanged => {
                Self::Returned(SubmissionReturnReason::PreflightChanged)
            }
            OutcomeDocument::KeyChecked => {
                Self::Paused(tallyline_submission::api::model::SubmissionPauseReason::KeyChecked)
            }
            OutcomeDocument::Reauthenticate => Self::Paused(
                tallyline_submission::api::model::SubmissionPauseReason::Reauthenticate,
            ),
            OutcomeDocument::KeyCheckUnavailable => Self::Paused(
                tallyline_submission::api::model::SubmissionPauseReason::KeyCheckUnavailable,
            ),
            OutcomeDocument::MaxRowsExceeded => {
                Self::Returned(SubmissionReturnReason::MaxRowsExceeded)
            }
            OutcomeDocument::TerminalNotFound => {
                Self::Returned(SubmissionReturnReason::TerminalNotFound)
            }
            OutcomeDocument::InvalidScreenshot => {
                Self::Returned(SubmissionReturnReason::InvalidScreenshot)
            }
            OutcomeDocument::ScreenshotTooLarge => {
                Self::Returned(SubmissionReturnReason::ScreenshotTooLarge)
            }
            OutcomeDocument::OutcomeCheckRequired => Self::OutcomeCheckRequired,
            OutcomeDocument::RateLimited => Self::NotProcessed(NotProcessed::RateLimited),
            OutcomeDocument::RowBudget => Self::NotProcessed(NotProcessed::RowBudget),
            OutcomeDocument::ImageUpload => Self::NotProcessed(NotProcessed::ImageUpload),
            OutcomeDocument::ImageStorage => Self::NotProcessed(NotProcessed::ImageStorage),
            OutcomeDocument::DailyQuota => Self::NotProcessed(NotProcessed::DailyQuota),
            OutcomeDocument::UpdateRequired => Self::Paused(
                tallyline_submission::api::model::SubmissionPauseReason::UpdateRequired,
            ),
            OutcomeDocument::DuplicateCooldown => {
                Self::Held(tallyline_submission::api::model::SubmissionHoldReason::Cooldown)
            }
            OutcomeDocument::PossiblyAlreadyReceived => {
                Self::Returned(SubmissionReturnReason::PossiblyAlreadyReceived)
            }
            OutcomeDocument::Unknown => Self::Unknown,
            OutcomeDocument::Rejected => Self::Rejected,
            OutcomeDocument::AppToken => {
                Self::Paused(tallyline_submission::api::model::SubmissionPauseReason::AppToken)
            }
            OutcomeDocument::AcceptanceClosed => {
                Self::Held(tallyline_submission::api::model::SubmissionHoldReason::AcceptanceClosed)
            }
            OutcomeDocument::ApiChanged => {
                Self::Held(tallyline_submission::api::model::SubmissionHoldReason::ApiChanged)
            }
            OutcomeDocument::DryRunSucceeded => Self::DryRunSucceeded,
            OutcomeDocument::Partial => Self::Partial,
            OutcomeDocument::ConnectFailure => Self::NotProcessed(NotProcessed::ConnectFailure),
            OutcomeDocument::ServiceUnavailable => {
                Self::NotProcessed(NotProcessed::ServiceUnavailable)
            }
            OutcomeDocument::ScreenshotRequired => {
                Self::Returned(SubmissionReturnReason::ScreenshotRequired)
            }
            OutcomeDocument::InvalidObservationTime => {
                Self::Returned(SubmissionReturnReason::InvalidObservationTime)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorrectionDocument {
    ReviewClient,
    NewCapture,
}
impl From<SubmissionCorrection> for CorrectionDocument {
    fn from(value: SubmissionCorrection) -> Self {
        match value {
            SubmissionCorrection::ReviewClient => Self::ReviewClient,
            SubmissionCorrection::NewCapture => Self::NewCapture,
        }
    }
}
impl From<CorrectionDocument> for SubmissionCorrection {
    fn from(value: CorrectionDocument) -> Self {
        match value {
            CorrectionDocument::ReviewClient => Self::ReviewClient,
            CorrectionDocument::NewCapture => Self::NewCapture,
        }
    }
}
