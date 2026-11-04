// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Closed account-check and originating-attempt evidence documents.

use crate::queue_outcome_document::ResultDocument;
use serde::{Deserialize, Serialize};
use tallyline_shared_kernel::api::model::Timestamp;
use tallyline_submission::api::model::{
    SubmissionAccountCheck, SubmissionAccountEvidence, SubmissionPauseReason,
    SubmissionRepositoryError, SubmissionRequest, SubmissionResult, UexErrorCode,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AccountCheckDocument {
    NotRequired,
    Pending,
    Completed(Box<ResultDocument>),
}
impl From<SubmissionAccountCheck> for AccountCheckDocument {
    fn from(value: SubmissionAccountCheck) -> Self {
        match value {
            SubmissionAccountCheck::NotRequired => Self::NotRequired,
            SubmissionAccountCheck::Pending => Self::Pending,
            SubmissionAccountCheck::Completed(reason) => Self::Completed(Box::new(
                ResultDocument::from(&SubmissionResult::Paused(reason)),
            )),
        }
    }
}
impl AccountCheckDocument {
    pub(crate) fn restore(
        self,
        request: &SubmissionRequest,
        sent: Option<Timestamp>,
    ) -> Result<SubmissionAccountCheck, SubmissionRepositoryError> {
        Ok(match self {
            Self::NotRequired => SubmissionAccountCheck::NotRequired,
            Self::Pending => SubmissionAccountCheck::Pending,
            Self::Completed(reason) => {
                SubmissionAccountCheck::Completed(action(*reason, request, sent)?)
            }
        })
    }
}
fn action(
    value: ResultDocument,
    request: &SubmissionRequest,
    sent: Option<Timestamp>,
) -> Result<SubmissionPauseReason, SubmissionRepositoryError> {
    let SubmissionResult::Paused(reason) = value.restore(request, sent)? else {
        return Err(SubmissionRepositoryError::Invalid);
    };
    Ok(reason)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AccountEvidenceDocument {
    code: Option<String>,
    http_status: u16,
    original_action: ResultDocument,
    check: AccountCheckDocument,
    origin_revision: Option<u64>,
}
impl From<&SubmissionAccountEvidence> for AccountEvidenceDocument {
    fn from(value: &SubmissionAccountEvidence) -> Self {
        Self {
            code: value.code().map(|code| code.as_str().to_owned()),
            http_status: value.http_status(),
            original_action: ResultDocument::from(&SubmissionResult::Paused(
                value.original_action(),
            )),
            check: value.check().into(),
            origin_revision: value.origin_revision(),
        }
    }
}
impl AccountEvidenceDocument {
    pub(crate) fn restore(
        self,
        request: &SubmissionRequest,
        sent: Option<Timestamp>,
    ) -> Result<SubmissionAccountEvidence, SubmissionRepositoryError> {
        let code = match self.code {
            Some(code) => Some(UexErrorCode::new(code).ok_or(SubmissionRepositoryError::Invalid)?),
            None => None,
        };
        Ok(SubmissionAccountEvidence::new(
            code,
            self.http_status,
            action(self.original_action, request, sent)?,
            self.check.restore(request, sent)?,
            self.origin_revision,
        ))
    }
}
