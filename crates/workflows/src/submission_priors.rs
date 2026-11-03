// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Accepted production outcomes mapped to explicit local prior records.

use std::{num::NonZeroU8, sync::Arc};
use tallyline_reference_data::api::model::{
    AcceptedLocalPrior, AcceptedPriorContext, InvalidPrior, PriorValues,
};
use tallyline_shared_kernel::api::model::SubmissionMode;
use tallyline_submission::api::model::{JobPhase, SubmissionJob, SubmissionResult};

/// Produces only accepted production records for the workflow's atomic outcome transaction.
///
/// Unmapped partial rows and unknown outcomes never change a prior. A full-count successful
/// response establishes acceptance even while its returned identities remain unverified. PTU
/// records remain history data; the reference lookup never promotes them to usable PTU prices.
///
/// # Errors
/// Refuses inconsistent success metadata instead of omitting a supposedly accepted row.
pub fn accepted_submission_priors(
    job: &SubmissionJob,
) -> Result<Vec<AcceptedLocalPrior>, InvalidPrior> {
    let request = job.request();
    let header = request.header();
    if header.mode != SubmissionMode::Production || job.phase() != JobPhase::Succeeded {
        return Ok(Vec::new());
    }
    if !matches!(
        job.result(),
        Some(SubmissionResult::ProductionSucceeded(_) | SubmissionResult::ProductionAccepted(_))
    ) {
        return Err(InvalidPrior);
    }
    let accepted_at = job
        .attempts()
        .last()
        .and_then(|attempt| attempt.ended_at())
        .ok_or(InvalidPrior)?;
    let context = Arc::new(AcceptedPriorContext {
        report: header.report,
        report_version: header.report_version,
        submission: job.id(),
        environment: header.environment,
        terminal: header.terminal,
        side: header.side,
        game_version: header.game_version.clone(),
        observed_at: header.observed_at,
        accepted_at,
    });
    request
        .rows()
        .iter()
        .map(|row| {
            AcceptedLocalPrior::new(
                Arc::clone(&context),
                row.commodity(),
                PriorValues {
                    price: row.price(),
                    stock: row.stock(),
                    status: Some(NonZeroU8::new(row.status()).ok_or(InvalidPrior)?),
                    container_sizes: Vec::new(),
                },
            )
        })
        .collect()
}
