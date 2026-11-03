// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fixed-mode queue presentation and version-bound cancellation intent.

use crate::api::Catalogue;
use tallyline_shared_kernel::api::model::SubmissionMode;
use tallyline_submission::api::model::{
    JobId, JobPhase, SubmissionApplicationError, SubmissionJob, SubmissionRepositoryError,
    SubmissionResult, VerifiedReceiptMapping, VerifiedUexReport,
};

/// An explicit queue action without credentials or mutable report values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueCommand {
    /// Cancels only the exact unsent job version shown to the user.
    Cancel {
        /// Stable local submission identity.
        id: JobId,
        /// The committed version observed before the action.
        expected_revision: u64,
    },
}

/// Current committed job views and one outstanding explicit cancellation.
#[derive(Default)]
pub struct SubmissionQueue {
    jobs: Vec<SubmissionJob>,
    mappings: Vec<VerifiedReceiptMapping>,
    waiting: Vec<tallyline_submission::api::model::SubmissionWait>,
    pending: Option<QueueCommand>,
    busy: Option<QueueCommand>,
    failure: Option<SubmissionApplicationError>,
}
impl SubmissionQueue {
    /// Retains the newest committed version of every received job.
    pub fn received(&mut self, jobs: Vec<SubmissionJob>) {
        for job in jobs {
            if let Some(old) = self.jobs.iter_mut().find(|old| old.id() == job.id()) {
                if job.revision() > old.revision() {
                    *old = job;
                }
            } else {
                self.jobs.push(job);
            }
        }
        self.mappings
            .retain(|mapping| self.jobs.iter().any(|job| mapping.matches_job(job)));
        self.jobs.sort_by_key(|job| std::cmp::Reverse(job.id()));
        if let Some(QueueCommand::Cancel {
            id,
            expected_revision,
        }) = self.busy
            && self
                .jobs
                .iter()
                .any(|job| job.id() == id && job.revision() > expected_revision)
        {
            self.busy = None;
            self.pending = None;
            self.failure = None;
        }
    }
    /// Retains waits only for the exact committed queued revision currently visible.
    pub fn received_waiting(
        &mut self,
        waiting: Vec<tallyline_submission::api::model::SubmissionWait>,
    ) {
        self.waiting = waiting
            .into_iter()
            .filter(|wait| {
                self.jobs.iter().any(|job| {
                    job.id() == wait.job
                        && job.revision() == wait.revision
                        && job.phase() == JobPhase::Queued
                })
            })
            .collect();
    }
    /// Publishes a persisted mapping only for the exact committed successful job already visible.
    pub fn received_mapping(&mut self, mapping: VerifiedReceiptMapping) -> bool {
        if !self.jobs.iter().any(|job| mapping.matches_job(job)) {
            return false;
        }
        if let Some(previous) = self
            .mappings
            .iter()
            .find(|previous| previous.job() == mapping.job())
        {
            return previous == &mapping;
        }
        self.mappings.push(mapping);
        true
    }
    /// Borrows verified identities only while their exact successful snapshot remains current.
    #[must_use]
    pub fn verified_receipts(&self, id: JobId) -> Option<&[VerifiedUexReport]> {
        let job = self.jobs.iter().find(|job| job.id() == id)?;
        self.mappings
            .iter()
            .find(|mapping| mapping.matches_job(job))
            .map(VerifiedReceiptMapping::receipts)
    }
    /// Borrows committed jobs, newest first, without inventing intermediate phases.
    #[must_use]
    pub fn jobs(&self) -> &[SubmissionJob] {
        &self.jobs
    }
    /// Returns whether this view can request cancellation of the current unsent job.
    #[must_use]
    pub fn can_cancel(&self, id: JobId) -> bool {
        self.busy.is_none()
            && self.jobs.iter().any(|job| {
                job.id() == id && matches!(job.phase(), JobPhase::Queued | JobPhase::Held)
            })
    }
    /// Queues at most one explicit version-bound cancellation.
    pub fn request_cancel(&mut self, id: JobId) -> bool {
        if !self.can_cancel(id) {
            return false;
        }
        let Some(job) = self.jobs.iter().find(|job| job.id() == id) else {
            return false;
        };
        let command = QueueCommand::Cancel {
            id,
            expected_revision: job.revision(),
        };
        self.pending = Some(command);
        self.busy = Some(command);
        self.failure = None;
        true
    }
    /// Drains the bounded command for the application worker.
    pub fn take_command(&mut self) -> Option<QueueCommand> {
        self.pending.take()
    }
    /// Applies a passive failure only to the still outstanding exact command.
    pub fn failed(
        &mut self,
        id: JobId,
        expected_revision: u64,
        failure: SubmissionApplicationError,
    ) {
        if self.busy
            == Some(QueueCommand::Cancel {
                id,
                expected_revision,
            })
        {
            self.busy = None;
            self.pending = None;
            self.failure = Some(failure);
        }
    }
}

/// Draws immutable mode and account context and emits only explicit unsent cancellation intent.
pub fn show_submission_queue(
    ui: &mut egui::Ui,
    state: &mut SubmissionQueue,
    catalogue: &Catalogue,
    theme: tallyline_egui::Theme,
) -> Option<QueueCommand> {
    ui.heading(catalogue.text("queue.title"));
    if state.jobs.is_empty() {
        ui.label(catalogue.text("queue.empty"));
    }
    if let Some(failure) = state.failure {
        let key = match failure {
            SubmissionApplicationError::Missing => "queue.missing",
            SubmissionApplicationError::Refused(_) => "queue.cannot_cancel",
            SubmissionApplicationError::Repository(SubmissionRepositoryError::Conflict) => {
                "queue.changed"
            }
            SubmissionApplicationError::Repository(
                SubmissionRepositoryError::Invalid | SubmissionRepositoryError::Unavailable,
            ) => "queue.unavailable",
        };
        ui.label(catalogue.text(key));
    }
    let mut cancel = None;
    for job in &state.jobs {
        ui.push_id(job.id().0, |ui| {
            let header = job.request().header();
            ui.group(|ui| {
                ui.label(format!(
                    "{} {} · {} {} · {}",
                    catalogue.text("queue.report"),
                    header.report.0,
                    catalogue.text("queue.submission"),
                    job.id().0,
                    header.account
                ));
                show_submission_mode(ui, header.mode, catalogue, theme);
                ui.label(catalogue.text(phase_key(job)));
                if let Some(wait) = state.waiting.iter().find(|wait| {
                    wait.job == job.id()
                        && wait.revision == job.revision()
                        && job.phase() == JobPhase::Queued
                }) {
                    crate::submission_waiting::show(ui, wait, catalogue);
                }
                ui.label(format!(
                    "{} {} · {} {} · {} {}",
                    catalogue.text("queue.observed"),
                    crate::attachment_views::format_time(header.observed_at),
                    job.request().rows().len(),
                    catalogue.text("queue.rows"),
                    job.attempts().len(),
                    catalogue.text("queue.attempts")
                ));
                if let Some(reason) = job.review_reason() {
                    ui.label(catalogue.text(review_reason_key(reason)));
                }
                show_result(ui, job, state.verified_receipts(job.id()), catalogue);
                if ui
                    .add_enabled(
                        state.can_cancel(job.id()),
                        egui::Button::new(format!(
                            "{} {}",
                            catalogue.text("queue.cancel"),
                            job.id().0
                        )),
                    )
                    .clicked()
                {
                    cancel = Some(job.id());
                }
            });
        });
    }
    if let Some(id) = cancel {
        state.request_cancel(id);
    }
    state.take_command()
}

/// Draws the fixed submission mode with its dedicated design-system tokens.
pub fn show_submission_mode(
    ui: &mut egui::Ui,
    mode: SubmissionMode,
    catalogue: &Catalogue,
    theme: tallyline_egui::Theme,
) {
    let palette = match theme {
        tallyline_egui::Theme::Dark => tallyline_egui::tokens::DARK,
        tallyline_egui::Theme::Light => tallyline_egui::tokens::LIGHT,
        tallyline_egui::Theme::HighContrast => tallyline_egui::tokens::HIGH_CONTRAST,
    };
    let (key, foreground, background) = match mode {
        SubmissionMode::DryRun => ("queue.dry_run", "env-test-fg", "env-test-bg"),
        SubmissionMode::Production => {
            ("queue.production", "env-production-fg", "env-production-bg")
        }
    };
    ui.add(tallyline_egui::Chip {
        text: catalogue.text(key),
        foreground: palette
            .color(foreground)
            .unwrap_or_else(|| ui.visuals().text_color()),
        background: palette
            .color(background)
            .unwrap_or(egui::Color32::TRANSPARENT),
    });
}
fn phase_key(job: &SubmissionJob) -> &'static str {
    match job.phase() {
        JobPhase::Queued => "queue.queued",
        JobPhase::Sending => "queue.sending",
        JobPhase::Held => "queue.held",
        JobPhase::Succeeded => match job.request().header().mode {
            SubmissionMode::DryRun => "queue.test_succeeded",
            SubmissionMode::Production => "queue.submitted",
        },
        JobPhase::PartiallyAccepted => "queue.partial",
        JobPhase::OutcomeUnknown => "queue.unknown",
        JobPhase::Returned => "queue.returned",
        JobPhase::Cancelled => "queue.cancelled",
    }
}
fn show_result(
    ui: &mut egui::Ui,
    job: &SubmissionJob,
    verified: Option<&[VerifiedUexReport]>,
    catalogue: &Catalogue,
) {
    if let Some(receipts) = verified {
        show_verified(ui, receipts, catalogue);
        return;
    }
    let Some(result) = job.result() else {
        return;
    };
    match result {
        SubmissionResult::DryRunSucceeded => {
            ui.label(catalogue.text("queue.no_uex_ids"));
        }
        SubmissionResult::ProductionSucceeded(receipts) => {
            show_verified(ui, receipts, catalogue);
        }
        SubmissionResult::ProductionAccepted(ids) | SubmissionResult::Partial(ids) => {
            for id in ids {
                ui.label(format!(
                    "{} {} · {}",
                    catalogue.text("queue.unverified_id"),
                    id.id(),
                    crate::attachment_views::format_time(id.date_added())
                ));
            }
        }
        SubmissionResult::Returned(
            tallyline_submission::api::model::SubmissionReturnReason::PreflightChanged,
        ) => {
            ui.label(catalogue.text("queue.preflight_changed"));
        }
        SubmissionResult::Unknown => {
            ui.label(catalogue.text("queue.no_replay"));
        }
        SubmissionResult::Returned(
            tallyline_submission::api::model::SubmissionReturnReason::ScreenshotRequired,
        ) => {
            ui.label(catalogue.text("queue.screenshot_required"));
        }
        SubmissionResult::Returned(
            tallyline_submission::api::model::SubmissionReturnReason::InvalidObservationTime,
        ) => {
            ui.label(catalogue.text("queue.invalid_observation_time"));
        }
        SubmissionResult::NotProcessed(
            tallyline_submission::api::model::NotProcessed::ConnectFailure,
        ) => {
            ui.label(catalogue.text("queue.connect_failure"));
        }
    }
}
fn show_verified(ui: &mut egui::Ui, receipts: &[VerifiedUexReport], catalogue: &Catalogue) {
    for receipt in receipts {
        ui.label(format!(
            "{} {} · {} {}",
            catalogue.text("queue.verified_id"),
            receipt.id(),
            catalogue.text("queue.row"),
            receipt.row().0
        ));
    }
}

pub(crate) fn review_reason_key(
    reason: tallyline_submission::api::model::SendReturnReason,
) -> &'static str {
    use tallyline_submission::api::model::SendReturnReason;
    match reason {
        SendReturnReason::ReviewRequired => "queue.review_required",
        SendReturnReason::GameVersionChanged => "queue.version_changed",
        SendReturnReason::ObservationAgeUnconfirmed => "queue.age_unconfirmed",
        SendReturnReason::ObservationTooOld => "queue.observation_too_old",
        SendReturnReason::ObservationInFuture => "queue.observation_in_future",
    }
}
