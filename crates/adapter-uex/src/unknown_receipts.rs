// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded positive-only receipt reads for an uncertain production attempt.

use crate::api::UexReader;
use serde_json::Value;
use std::collections::BTreeSet;
use tallyline_shared_kernel::api::model::SubmissionMode;
use tallyline_submission::api::SubmissionUnknownReceiptVerifier;
use tallyline_submission::api::model::{
    CredentialValues, JobPhase, SubmissionJob, VerifiedUexReport,
};

impl SubmissionUnknownReceiptVerifier for UexReader {
    async fn verify_unknown_receipts(
        &self,
        job: &SubmissionJob,
        credentials: &CredentialValues,
        release_token: Option<&str>,
    ) -> Option<Vec<VerifiedUexReport>> {
        if job.phase() != JobPhase::OutcomeUnknown
            || job.request().header().mode != SubmissionMode::Production
        {
            return None;
        }
        let date_added = job.attempts().last()?.date_added();
        let token = credentials.own_token().or(release_token)?;
        let bearer = format!("Bearer {token}");
        let request = job.request();
        let terminal = request.header().terminal.0.to_string();
        let verification = async {
            let data = self
                .transport
                .get(
                    "data_info/",
                    &[
                        ("type", "commodity"),
                        ("id_terminal", &terminal),
                        ("username", &request.header().account),
                        ("limit", "100"),
                    ],
                    &[
                        ("Authorization", &bearer),
                        ("secret-key", credentials.secret_key()),
                    ],
                )
                .await;
            let Ok(Value::Array(records)) = data else {
                return None;
            };
            let snapshot = request.clone();
            self.transport
                .process(move || {
                    if !crate::receipt_window::valid(&records, &snapshot) {
                        return Ok(None);
                    }
                    let mut receipts = Vec::new();
                    let mut rows = BTreeSet::new();
                    let mut identities = BTreeSet::new();
                    for record in &records {
                        if let Some(receipt) = record
                            .get("id")
                            .and_then(crate::submission::number)
                            .and_then(|id| {
                                crate::submission::lookup(record, id, &snapshot, date_added)
                            })
                        {
                            if !rows.insert(receipt.row().0) || !identities.insert(receipt.id()) {
                                return Ok(None);
                            }
                            receipts.push(receipt);
                        }
                    }
                    Ok(Some(receipts))
                })
                .await
                .ok()
                .flatten()
        };
        tokio::time::timeout(crate::receipt_window::TIMEOUT, verification)
            .await
            .ok()
            .flatten()
    }
}
