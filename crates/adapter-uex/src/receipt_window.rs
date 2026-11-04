// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared bounded data-info window validation.

use serde_json::Value;
use std::time::Duration;
use tallyline_submission::api::model::SubmissionRequest;

pub(crate) const LIMIT: usize = 100;
pub(crate) const TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) fn valid(records: &[Value], request: &SubmissionRequest) -> bool {
    !(records.len() > LIMIT
        || records.iter().any(|record| {
            record.get("type").and_then(Value::as_str) != Some("commodity")
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
        }))
}
