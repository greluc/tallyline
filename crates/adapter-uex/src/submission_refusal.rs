// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Durable-first account refusal and independent read-only receipt recovery.

use crate::api::{RejectedData, UexReader};
use tallyline_shared_kernel::api::model::{SubmissionMode, Timestamp};
use tallyline_submission::api::model::{
    CredentialValues, JobPhase, NotProcessed, SubmissionAccountCheck, SubmissionHoldReason,
    SubmissionJob, SubmissionPauseReason, SubmissionRequest, SubmissionResult,
    SubmissionReturnReason, UexErrorCode,
};
use tallyline_submission::api::model::{SubmissionCorrection, SubmissionRejection};

pub(crate) struct RejectedSubmission {
    pub(crate) http_status: u16,
    pub(crate) code: String,
    pub(crate) data: Option<RejectedData>,
}
#[derive(Clone, Copy)]
pub(crate) struct RejectionContext<'a> {
    pub(crate) request: &'a SubmissionRequest,
    pub(crate) date_added: Timestamp,
    pub(crate) credentials: &'a CredentialValues,
    pub(crate) token: &'a str,
}
impl UexReader {
    /// Repeats only a durably retained account or receipt check without a POST.
    pub async fn recheck_submission_refusal(
        &self,
        job: &SubmissionJob,
        credentials: &CredentialValues,
        release_token: Option<&str>,
    ) -> Option<SubmissionResult> {
        let result = job.result()?;
        let pending_receipt = matches!(
            result.final_result(),
            SubmissionResult::NeedsOutcomeCheck { .. }
        );
        if result.account_evidence().is_some_and(|evidence| {
            matches!(evidence.check(), SubmissionAccountCheck::Pending)
                || matches!(
                    evidence.check(),
                    SubmissionAccountCheck::Completed(SubmissionPauseReason::KeyCheckUnavailable)
                ) && !pending_receipt
        }) {
            let action = self
                .recheck_submission_account(credentials, &job.request().header().account)
                .await;
            return job.resolve_account_check(action).ok()?.result().cloned();
        }
        let SubmissionResult::NeedsOutcomeCheck { code, http_status } = result.final_result()
        else {
            return None;
        };
        if job.phase() != JobPhase::Held {
            return None;
        }
        let token = credentials.own_token().or(release_token)?;
        let outcome = self
            .classify_refusal(
                *http_status,
                None,
                RejectionContext {
                    request: job.request(),
                    date_added: job.attempts().last()?.date_added(),
                    credentials,
                    token,
                },
                code.clone(),
            )
            .await;
        if matches!(
            outcome.final_result(),
            SubmissionResult::NeedsOutcomeCheck { .. }
        ) {
            return None;
        }
        Some(if let Some(evidence) = result.account_evidence() {
            let action = result.account_pause()?;
            let outcome = match outcome {
                SubmissionResult::Paused(_) => SubmissionResult::Paused(action),
                SubmissionResult::PausedCode { code, .. } => SubmissionResult::PausedCode {
                    code,
                    reason: action,
                },
                other @ (SubmissionResult::AccountRefusal { .. }
                | SubmissionResult::DuplicateEvidence(_)
                | SubmissionResult::RateLimitedCode(_)
                | SubmissionResult::Rechecked { .. }
                | SubmissionResult::PartialError { .. }
                | SubmissionResult::NeedsOutcomeCheck { .. }
                | SubmissionResult::Rejected(_)
                | SubmissionResult::ProductionSucceeded(_)
                | SubmissionResult::ProductionAccepted(_)
                | SubmissionResult::DryRunSucceeded
                | SubmissionResult::Partial(_)
                | SubmissionResult::NotProcessed(_)
                | SubmissionResult::VersionLockedOut(_)
                | SubmissionResult::Held(_)
                | SubmissionResult::Returned(_)
                | SubmissionResult::Unknown) => other,
            };
            SubmissionResult::AccountRefusal {
                code: evidence.code().cloned(),
                http_status: evidence.http_status(),
                reason: evidence.original_action(),
                check: evidence.check(),
                outcome: Box::new(outcome),
            }
        } else {
            outcome
        })
    }
    pub(crate) async fn submission_refusal(
        &self,
        error: RejectedSubmission,
        context: RejectionContext<'_>,
    ) -> SubmissionResult {
        let account = matches!(
            error.code.as_str(),
            "access_denied"
                | "no_api_found"
                | "not_allowed"
                | "missing_secret_key"
                | "invalid_secret_key"
                | "user_not_found"
                | "user_not_allowed"
                | "user_disabled"
                | "user_not_verified"
        ) || matches!(error.http_status, 401 | 403) && !known_code(&error.code);
        let code = if error.code.contains(context.credentials.secret_key())
            || error.code.contains(context.token)
        {
            None
        } else {
            UexErrorCode::new(error.code)
        };
        if !account {
            return match code {
                Some(code) => {
                    self.classify_refusal(error.http_status, error.data, context, code)
                        .await
                }
                None => SubmissionResult::Unknown,
            };
        }
        let token = code.as_ref().is_some_and(|code| {
            matches!(
                code.as_str(),
                "access_denied" | "no_api_found" | "not_allowed"
            )
        });
        let reason = if token {
            SubmissionPauseReason::AppToken
        } else {
            SubmissionPauseReason::KeyCheckUnavailable
        };
        let check = if token {
            SubmissionAccountCheck::NotRequired
        } else {
            SubmissionAccountCheck::Pending
        };
        let outcome = if context.request.header().mode == SubmissionMode::Production {
            match &code {
                None => SubmissionResult::Unknown,
                Some(code) if !before_storage(code.as_str()) => {
                    SubmissionResult::NeedsOutcomeCheck {
                        code: code.clone(),
                        http_status: error.http_status,
                    }
                }
                Some(_) => SubmissionResult::Paused(reason),
            }
        } else {
            SubmissionResult::Paused(reason)
        };
        SubmissionResult::AccountRefusal {
            code,
            http_status: error.http_status,
            reason,
            check,
            outcome: Box::new(outcome),
        }
    }
    async fn classify_refusal(
        &self,
        http_status: u16,
        data: Option<RejectedData>,
        context: RejectionContext<'_>,
        code: UexErrorCode,
    ) -> SubmissionResult {
        if code.as_str() == "error" && http_status >= 500 {
            return SubmissionResult::Unknown;
        }
        if http_status == 429 && !known_code(code.as_str()) {
            return SubmissionResult::RateLimitedCode(code);
        }
        if context.request.header().mode == SubmissionMode::Production
            && !before_storage(code.as_str())
        {
            let row = match data {
                Some(RejectedData::Row(row)) if row < context.request.rows().len() => Some(row),
                Some(RejectedData::Report {
                    id_terminal,
                    id_commodity,
                }) if id_terminal == u64::from(context.request.header().terminal.0) => context
                    .request
                    .rows()
                    .iter()
                    .position(|row| u64::from(row.commodity().0) == id_commodity),
                _ => None,
            };
            if row.is_some_and(|row| row > 0)
                || matches!(
                    code.as_str(),
                    "database_error" | "invalid_prices_array_format" | "reference_key_not_supplied"
                )
            {
                return SubmissionResult::PartialError { code, row };
            }
            if row.is_none() {
                match self.check_refusal_receipt(&context).await {
                    Some(true) => return SubmissionResult::PartialError { code, row: None },
                    Some(false) => {}
                    None => {
                        return SubmissionResult::NeedsOutcomeCheck { code, http_status };
                    }
                }
            }
        }
        if matches!(http_status, 401 | 403) && !known_code(code.as_str()) {
            return SubmissionResult::PausedCode {
                code,
                reason: SubmissionPauseReason::KeyCheckUnavailable,
            };
        }
        match code.as_str() {
            "duplicated_report" => SubmissionResult::Held(SubmissionHoldReason::Cooldown),
            "image_upload_error" => SubmissionResult::NotProcessed(NotProcessed::ImageUpload),
            "image_storage_error" => SubmissionResult::NotProcessed(NotProcessed::ImageStorage),
            "requests_limit_reached" => SubmissionResult::NotProcessed(NotProcessed::RateLimited),
            "too_many_reports" => SubmissionResult::NotProcessed(NotProcessed::RowBudget),
            "exceeded_daily_quota" => SubmissionResult::NotProcessed(NotProcessed::DailyQuota),
            "upgrade_required" => SubmissionResult::Paused(SubmissionPauseReason::UpdateRequired),
            "service_unavailable" => {
                SubmissionResult::NotProcessed(NotProcessed::ServiceUnavailable)
            }
            "access_denied" | "no_api_found" | "not_allowed" => {
                SubmissionResult::Paused(SubmissionPauseReason::AppToken)
            }
            "missing_secret_key" | "invalid_secret_key" | "user_not_found" | "user_not_allowed"
            | "user_disabled" | "user_not_verified" => {
                SubmissionResult::Paused(SubmissionPauseReason::KeyCheckUnavailable)
            }
            "ptu_reports_not_allowed" | "type_not_available" => {
                SubmissionResult::Held(SubmissionHoldReason::AcceptanceClosed)
            }
            "screenshot_required" => {
                SubmissionResult::Returned(SubmissionReturnReason::ScreenshotRequired)
            }
            "invalid_date" => {
                SubmissionResult::Returned(SubmissionReturnReason::InvalidObservationTime)
            }
            "max_rows_exceeded" => {
                SubmissionResult::Returned(SubmissionReturnReason::MaxRowsExceeded)
            }
            "terminal_not_found" => {
                SubmissionResult::Returned(SubmissionReturnReason::TerminalNotFound)
            }
            "invalid_screenshot" => {
                SubmissionResult::Returned(SubmissionReturnReason::InvalidScreenshot)
            }
            "screenshot_length_exceeds_limit" => {
                SubmissionResult::Returned(SubmissionReturnReason::ScreenshotTooLarge)
            }
            "invalid_game_version" => SubmissionResult::Rejected(SubmissionRejection::new(
                code,
                SubmissionCorrection::NewCapture,
            )),
            _ => SubmissionResult::Rejected(SubmissionRejection::new(
                code,
                SubmissionCorrection::ReviewClient,
            )),
        }
    }
    async fn check_refusal_receipt(&self, context: &RejectionContext<'_>) -> Option<bool> {
        let bearer = format!("Bearer {}", context.token);
        let terminal = context.request.header().terminal.0.to_string();
        let data = self
            .transport
            .get(
                "data_info/",
                &[
                    ("type", "commodity"),
                    ("id_terminal", &terminal),
                    ("username", &context.request.header().account),
                    ("limit", "100"),
                ],
                &[
                    ("Authorization", &bearer),
                    ("secret-key", context.credentials.secret_key()),
                ],
            )
            .await;
        let Ok(serde_json::Value::Array(records)) = data else {
            return None;
        };
        let request = context.request.clone();
        let date_added = context.date_added;
        self.transport
            .process(move || {
                if records.len() > 100
                    || records.iter().any(|record| {
                        record.get("type").and_then(serde_json::Value::as_str) != Some("commodity")
                            || record
                                .get("id")
                                .and_then(crate::submission::number)
                                .is_none_or(|id| id == 0)
                            || record
                                .get("id_terminal")
                                .and_then(crate::submission::number)
                                != Some(u64::from(request.header().terminal.0))
                            || record
                                .get("id_commodity")
                                .and_then(crate::submission::number)
                                .is_none_or(|id| id == 0 || u32::try_from(id).is_err())
                            || record
                                .get("date_added")
                                .and_then(crate::submission::number)
                                .is_none()
                    })
                {
                    return Ok(None);
                }
                let found = records.iter().any(|record| {
                    record
                        .get("id")
                        .and_then(crate::submission::number)
                        .and_then(|id| crate::submission::lookup(record, id, &request, date_added))
                        .is_some()
                });
                Ok(if found {
                    Some(true)
                } else if records.len() < 100 {
                    Some(false)
                } else {
                    None
                })
            })
            .await
            .ok()
            .flatten()
    }
}
fn before_storage(code: &str) -> bool {
    matches!(
        code,
        "access_denied"
            | "exceeded_daily_quota"
            | "missing_client_version"
            | "upgrade_required"
            | "service_unavailable"
            | "invalid_input"
            | "ptu_reports_not_allowed"
            | "invalid_game_version"
            | "invalid_date"
            | "missing_secret_key"
            | "invalid_secret_key"
            | "user_not_found"
            | "user_disabled"
            | "user_not_allowed"
            | "too_many_reports"
            | "requests_limit_reached"
            | "missing_type"
            | "invalid_type"
            | "type_not_available"
            | "missing_id_terminal"
            | "terminal_not_found"
            | "not_allowed_player_terminal"
            | "missing_prices_array"
            | "max_rows_exceeded"
            | "faction_affinity_under_minimum_range"
            | "faction_affinity_under_maximum_range"
            | "faction_affinity_not_allowed_for_current_type"
            | "invalid_screenshot"
            | "screenshot_length_exceeds_limit"
            | "screenshot_required"
            | "image_upload_error"
            | "image_storage_error"
    )
}

fn known_code(code: &str) -> bool {
    before_storage(code)
        || matches!(
            code,
            "no_api_found"
                | "not_allowed"
                | "user_not_verified"
                | "database_error"
                | "duplicated_report"
                | "invalid_prices_array"
                | "invalid_prices_array_format"
                | "reference_key_not_supplied"
                | "no_commodities_found"
                | "missing_id_commodity"
                | "invalid_id_commodity"
                | "has_no_prices_and_no_is_missing_set"
                | "has_both_price_buy_and_price_sell"
                | "has_both_scu_buy_and_scu_sell"
                | "cannot_have_both_status_buy_and_status_sell"
                | "cannot_have_both_price_buy_and_status_sell"
                | "cannot_have_both_price_buy_and_scu_sell"
                | "cannot_have_both_price_sell_and_status_buy"
                | "cannot_have_both_price_sell_and_scu_buy"
                | "invalid_status_buy"
                | "invalid_status_sell"
                | "invalid_quality"
        )
}
