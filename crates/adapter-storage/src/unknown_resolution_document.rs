// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typed persistence of explicit user statements about unknown delivery.

use serde::{Deserialize, Serialize};
use tallyline_shared_kernel::api::model::{ReportId, Timestamp};
use tallyline_submission::api::model::{
    SubmissionRepositoryError, SubmissionRequest, UnknownDecision, UnknownResolutionAction,
    VerifiedUexReport,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum UnknownActionDocument {
    Received {
        at: String,
    },
    NotReceived {
        at: String,
    },
    Duplicated {
        report: u64,
        at: String,
    },
    Verified {
        receipts: Vec<(u64, u64, String)>,
        at: String,
    },
}
impl From<&UnknownResolutionAction> for UnknownActionDocument {
    fn from(value: &UnknownResolutionAction) -> Self {
        match value {
            UnknownResolutionAction::Verified { receipts, at } => Self::Verified {
                receipts: receipts
                    .iter()
                    .map(|receipt| {
                        (
                            receipt.row().0,
                            receipt.id(),
                            receipt.date_added().millis().to_string(),
                        )
                    })
                    .collect(),
                at: at.millis().to_string(),
            },
            UnknownResolutionAction::Duplicated { report, at } => Self::Duplicated {
                report: report.0,
                at: at.millis().to_string(),
            },
            UnknownResolutionAction::Decision {
                decision: UnknownDecision::Received,
                at,
            } => Self::Received {
                at: at.millis().to_string(),
            },
            UnknownResolutionAction::Decision {
                decision: UnknownDecision::NotReceived,
                at,
            } => Self::NotReceived {
                at: at.millis().to_string(),
            },
        }
    }
}
impl UnknownActionDocument {
    pub(crate) fn restore(
        self,
        request: &SubmissionRequest,
        sent_at: Option<Timestamp>,
    ) -> Result<UnknownResolutionAction, SubmissionRepositoryError> {
        let (decision, at) = match self {
            Self::Verified { receipts, at } => {
                let sent_at = sent_at.ok_or(SubmissionRepositoryError::Invalid)?;
                let receipts = receipts
                    .into_iter()
                    .map(|(row, id, date)| {
                        let row = request
                            .rows()
                            .iter()
                            .find(|candidate| candidate.id().0 == row)
                            .ok_or(SubmissionRepositoryError::Invalid)?;
                        VerifiedUexReport::from_lookup_at(
                            id,
                            request.header().terminal,
                            row.commodity(),
                            crate::reference_cache::timestamp(&date)
                                .map_err(|_| SubmissionRepositoryError::Invalid)?,
                            request,
                            sent_at,
                        )
                        .map_err(|_| SubmissionRepositoryError::Invalid)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(UnknownResolutionAction::Verified {
                    receipts,
                    at: crate::reference_cache::timestamp(&at)
                        .map_err(|_| SubmissionRepositoryError::Invalid)?,
                });
            }
            Self::Duplicated { report, at } => {
                return Ok(UnknownResolutionAction::Duplicated {
                    report: ReportId(report),
                    at: crate::reference_cache::timestamp(&at)
                        .map_err(|_| SubmissionRepositoryError::Invalid)?,
                });
            }
            Self::Received { at } => (UnknownDecision::Received, at),
            Self::NotReceived { at } => (UnknownDecision::NotReceived, at),
        };
        let at = crate::reference_cache::timestamp(&at)
            .map_err(|_| SubmissionRepositoryError::Invalid)?;
        Ok(UnknownResolutionAction::Decision { decision, at })
    }
}
