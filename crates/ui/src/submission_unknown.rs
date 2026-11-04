// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Explicit unknown-resolution controls and user-decision provenance.

use crate::api::Catalogue;
use egui::{Button, Ui};
use tallyline_shared_kernel::api::model::SubmissionMode;
use tallyline_submission::api::model::{
    JobPhase, SubmissionJob, SubmissionResult, UnknownDecision, UnknownResolutionAction,
};

/// An explicit user action without permission to replay the original POST.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnknownQueueAction {
    /// Records a statement independently of positive remote proof.
    Decide(UnknownDecision),
    /// Reads bounded positive remote evidence for the original attempt.
    CheckReceipt,
    /// Creates a separate unreviewed Draft after not-received was recorded.
    DuplicateAsDraft,
}
impl UnknownQueueAction {
    pub(crate) fn eligible(self, job: &SubmissionJob) -> bool {
        if job.phase() != JobPhase::OutcomeUnknown {
            return false;
        }
        match self {
            Self::Decide(_) => job.unknown_actions().is_empty(),
            Self::CheckReceipt => job.request().header().mode == SubmissionMode::Production,
            Self::DuplicateAsDraft => job.can_duplicate_unknown(),
        }
    }
}

pub(crate) fn show(
    ui: &mut Ui,
    job: &SubmissionJob,
    history: &[SubmissionJob],
    catalogue: &Catalogue,
    enabled: bool,
) -> Option<UnknownQueueAction> {
    if job.phase() != JobPhase::OutcomeUnknown {
        return None;
    }
    crate::unknown_attempt::show(ui, job, catalogue);
    if let Some(decision) = job.unknown_decision() {
        ui.label(catalogue.text(match decision {
            UnknownDecision::Received => "queue.unknown_received",
            UnknownDecision::NotReceived => "queue.unknown_not_received",
        }));
    }
    let verified = job.unknown_receipts().count();
    if verified > 0 {
        ui.label(catalogue.text(if verified == job.request().rows().len() {
            "queue.unknown_verified_full"
        } else {
            "queue.unknown_partial"
        }));
        for receipt in job.unknown_receipts() {
            ui.label(format!(
                "{} {} · {} {} · {}",
                catalogue.text("queue.verified_row"),
                receipt.row().0,
                catalogue.text("queue.uex_report"),
                receipt.id(),
                crate::attachment_views::format_time(receipt.date_added())
            ));
        }
    }
    ui.label(catalogue.text("queue.unknown_withdrawal_unavailable"));
    for action in job.unknown_actions() {
        if let UnknownResolutionAction::Duplicated { report, .. } = action {
            ui.label(format!(
                "{} {}",
                catalogue.text("queue.unknown_new_draft"),
                report.0
            ));
        }
    }
    for candidate in history {
        if matches!(candidate.result().map(SubmissionResult::final_result), Some(SubmissionResult::DuplicateEvidence(previous)) if *previous == job.id())
        {
            ui.label(format!(
                "{} {}",
                catalogue.text("queue.unknown_duplicate_evidence"),
                candidate.id().0
            ));
        }
    }
    for (key, action) in [
        (
            "queue.mark_received",
            UnknownQueueAction::Decide(UnknownDecision::Received),
        ),
        (
            "queue.mark_not_received",
            UnknownQueueAction::Decide(UnknownDecision::NotReceived),
        ),
        ("queue.check_unknown", UnknownQueueAction::CheckReceipt),
        (
            "queue.duplicate_unknown",
            UnknownQueueAction::DuplicateAsDraft,
        ),
    ] {
        if action.eligible(job)
            && ui
                .add_enabled(
                    enabled,
                    Button::new(tallyline_egui::button_text(format!(
                        "{} {}",
                        catalogue.text(key),
                        job.id().0
                    ))),
                )
                .clicked()
        {
            return Some(action);
        }
    }
    None
}
