// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded UI control composed inside the actual session credential owner.

use crate::api::{CredentialQueue, SubmissionControlError, SubmissionQueueController};
use std::io;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use tallyline_shared_kernel::api::model::Timestamp;
use tallyline_submission::api::CredentialApplication;
use tallyline_submission::api::model::{AccountState, ReleaseToken};
use tallyline_ui::api::model::UnknownQueueAction;
use tallyline_ui::api::model::{
    BackgroundEvent, SubmissionControl, SubmissionFeedback, SubmissionIntent, SubmissionUpdate,
};
use tallyline_workflows::api::{OutboxEvent, OutboxKind};

/// Retains backpressure feedback while HTTP completion and durable save control keep progressing.
pub struct SubmissionQueueBridge {
    controller: SubmissionQueueController,
    incoming: Receiver<SubmissionControl>,
    outgoing: SyncSender<BackgroundEvent>,
    wake: Box<dyn Fn() + Send>,
    pending: Option<BackgroundEvent>,
    initial: bool,
    reference_publication: Option<Box<dyn FnMut() -> bool + Send>>,
    publication_pending: bool,
    notifications: Vec<OutboxEvent>,
    notification_published: bool,
    release_refresh: Option<Box<dyn crate::api::ReleaseReferenceRefresh>>,
    release_pending: Option<SubmissionControl>,
}
impl SubmissionQueueBridge {
    /// Composes bounded channels without creating a thread, runtime or credential copy.
    #[must_use]
    pub fn new(
        controller: SubmissionQueueController,
        incoming: Receiver<SubmissionControl>,
        outgoing: SyncSender<BackgroundEvent>,
        wake: impl Fn() + Send + 'static,
    ) -> Self {
        Self {
            controller,
            incoming,
            outgoing,
            wake: Box::new(wake),
            pending: None,
            initial: true,
            reference_publication: None,
            publication_pending: false,
            notifications: Vec::new(),
            notification_published: false,
            release_refresh: None,
            release_pending: None,
        }
    }
    /// Requires an owned reference-attempt acknowledgement before a manual or test release.
    /// Without this composition, release remains closed.
    #[must_use]
    pub fn with_release_refresh(
        mut self,
        refresh: impl crate::api::ReleaseReferenceRefresh + 'static,
    ) -> Self {
        self.release_refresh = Some(Box::new(refresh));
        self
    }
    fn begin_command(
        &mut self,
        command: SubmissionControl,
        application: &CredentialApplication,
        token: Option<&ReleaseToken>,
    ) -> Result<bool, SubmissionControlError> {
        if let SubmissionControl::Release {
            id,
            expected,
            account,
            ..
        } = &command
        {
            if application.release_account() != Some(account.as_str()) {
                return Err(SubmissionControlError::Conflict);
            }
            let report = self
                .controller
                .draft(*id)
                .map_err(|_| SubmissionControlError::Unavailable)?
                .ok_or(SubmissionControlError::Conflict)?;
            if report.version() != *expected {
                return Err(SubmissionControlError::Conflict);
            }
            self.release_refresh
                .as_mut()
                .ok_or(SubmissionControlError::Unavailable)?
                .begin(report.key().terminal())
                .map_err(|_| SubmissionControlError::Unavailable)?;
            self.release_pending = Some(command);
            Ok(false)
        } else {
            self.command(command, application, token).map(|()| true)
        }
    }
    fn finish_release(
        &mut self,
        application: &CredentialApplication,
    ) -> Result<bool, SubmissionControlError> {
        let Some(SubmissionControl::Release { account, .. }) = &self.release_pending else {
            return Ok(false);
        };
        if application.release_account() != Some(account.as_str()) {
            self.release_pending = None;
            return Err(SubmissionControlError::Conflict);
        }
        let result = self
            .release_refresh
            .as_mut()
            .ok_or(SubmissionControlError::Unavailable)?
            .completed()
            .map_err(|_| SubmissionControlError::Unavailable);
        match result {
            Ok(false) => Ok(false),
            Ok(true) => {
                let command = self
                    .release_pending
                    .take()
                    .ok_or(SubmissionControlError::Invalid)?;
                self.command(command, application, None).map(|()| true)
            }
            Err(error) => {
                self.release_pending = None;
                Err(error)
            }
        }
    }
    /// Signals cached-reference publication after committed queue progress without requesting GETs.
    /// A full publication channel retains the signal for a later bounded control step.
    #[must_use]
    pub fn with_reference_publication(
        mut self,
        publish: impl FnMut() -> bool + Send + 'static,
    ) -> Self {
        self.reference_publication = Some(Box::new(publish));
        self
    }
    fn publish_references(&mut self) {
        if self.publication_pending
            && self
                .reference_publication
                .as_mut()
                .is_some_and(|publish| publish())
        {
            self.publication_pending = false;
        }
    }
    fn load_notifications(&mut self) -> io::Result<bool> {
        if !self.notifications.is_empty() {
            return Ok(false);
        }
        self.notifications = self
            .controller
            .pending_events()
            .map_err(|_| io::Error::other("submission notifications unavailable"))?;
        self.notification_published = false;
        self.publication_pending |= self.notifications.iter().any(|event| {
            event.kind() == OutboxKind::SubmissionSucceeded
                && self.controller.jobs().iter().any(|job| {
                    job.id().0 == event.id().aggregate_id()
                        && job.phase() == tallyline_submission::api::model::JobPhase::Succeeded
                        && job.request().header().mode == tallyline_shared_kernel::api::model::SubmissionMode::Production
                        && job.request().header().environment == tallyline_shared_kernel::api::model::UexEnvironment::Live
                        && matches!(job.result(), Some(tallyline_submission::api::model::SubmissionResult::ProductionAccepted(_) | tallyline_submission::api::model::SubmissionResult::ProductionSucceeded(_)))
                })
        });
        Ok(!self.notifications.is_empty())
    }
    fn acknowledge_notifications(&mut self) -> io::Result<bool> {
        if !self.notification_published || self.publication_pending {
            return Ok(false);
        }
        for event in &self.notifications {
            self.controller.mark_delivered(*event).map_err(|_| {
                io::Error::other("submission notification acknowledgement unavailable")
            })?;
        }
        let delivered = !self.notifications.is_empty();
        self.notifications.clear();
        self.notification_published = false;
        Ok(delivered)
    }
    fn command(
        &mut self,
        command: SubmissionControl,
        application: &CredentialApplication,
        token: Option<&ReleaseToken>,
    ) -> Result<(), SubmissionControlError> {
        match command {
            SubmissionControl::Unknown {
                id,
                expected,
                action,
            } => match action {
                UnknownQueueAction::Decide(decision) => self
                    .controller
                    .decide_unknown(id, expected, decision)
                    .map(|_| ()),
                UnknownQueueAction::CheckReceipt => {
                    self.controller
                        .check_unknown(id, expected, application, token)
                }
                UnknownQueueAction::DuplicateAsDraft => {
                    self.controller.duplicate_unknown(id, expected).map(|_| ())
                }
            },
            SubmissionControl::Release {
                id,
                expected,
                account,
                intent,
            } => {
                if !matches!(application.state(), AccountState::Validated { username, .. } if username == &account)
                {
                    return Err(SubmissionControlError::Conflict);
                }
                match intent {
                    SubmissionIntent::Manual(mode) => {
                        self.controller
                            .release_manual(id, expected, mode, application)
                    }
                    SubmissionIntent::OnboardingTest => {
                        self.controller.release_test(id, expected, application)
                    }
                }
                .map(|_| ())
            }
            SubmissionControl::Retry { id, expected } => {
                self.controller.retry_now(id, expected).map(|_| ())
            }
            SubmissionControl::Cancel { id, expected } => {
                self.controller.cancel(id, expected).map(|_| ())
            }
            SubmissionControl::ConfirmNewer {
                id,
                expected,
                row,
                evidence,
            } => self
                .controller
                .confirm_newer_observation(id, expected, row, evidence, application)
                .map(|_| ()),
            SubmissionControl::ConfirmAge { id, expected } => self
                .controller
                .confirm_observation_age(id, expected)
                .map(|_| ()),
        }
    }
    fn publish(&mut self) -> io::Result<()> {
        let Some(event) = self.pending.take() else {
            return Ok(());
        };
        match self.outgoing.try_send(event) {
            Ok(()) => {
                self.notification_published = !self.notifications.is_empty();
                (self.wake)();
            }
            Err(TrySendError::Full(event)) => self.pending = Some(event),
            Err(TrySendError::Disconnected(_)) => {
                return Err(io::Error::other("submission view disconnected"));
            }
        }
        Ok(())
    }
}
impl CredentialQueue for SubmissionQueueBridge {
    fn poll(
        &mut self,
        application: &CredentialApplication,
        token: Option<&ReleaseToken>,
        _: Timestamp,
    ) -> io::Result<()> {
        self.publish()?;
        self.publish_references();
        let acknowledged = self.acknowledge_notifications()?;
        let mut changed = self.initial;
        self.initial = false;
        let mut feedback = None;
        let mut completed = false;
        let mut background_feedback = None;
        let background_changed;
        if self.pending.is_none()
            && self.release_pending.is_none()
            && let Ok(command) = self.incoming.try_recv()
        {
            changed = true;
            match self.begin_command(command, application, token) {
                Ok(done) => completed = done,
                Err(error) => {
                    completed = true;
                    feedback = Some(feedback_for(error));
                }
            }
        }
        match self.finish_release(application) {
            Ok(true) => {
                changed = true;
                completed = true;
            }
            Ok(false) => (),
            Err(error) => {
                changed = true;
                completed = true;
                feedback = Some(feedback_for(error));
            }
        }
        match self.controller.tick(application, token) {
            Ok(progress) => {
                changed |= progress;
                background_changed = progress;
            }
            Err(error) => {
                changed = true;
                background_changed = true;
                background_feedback = Some(feedback_for(error));
            }
        }
        if changed || acknowledged {
            changed |= self.load_notifications()?;
        }
        if changed {
            let reports = self
                .controller
                .drafts()
                .map_err(|_| io::Error::other("submission Draft snapshot unavailable"))?;
            if let Some(BackgroundEvent::SubmissionUpdated(previous)) = &self.pending {
                if previous.completed {
                    feedback = previous.feedback.clone();
                }
                if !background_changed {
                    background_feedback = previous.background_feedback.clone();
                }
                completed |= previous.completed;
            }
            self.pending = Some(BackgroundEvent::SubmissionUpdated(Box::new(
                SubmissionUpdate {
                    client_version: self.controller.client_version(),
                    service_fences: self.controller.service_fences().map_err(|_| {
                        io::Error::other("submission service-fence snapshot unavailable")
                    })?,
                    jobs: self.controller.jobs().to_vec(),
                    mappings: self.controller.mappings().to_vec(),
                    waiting: self.controller.waiting().to_vec(),
                    reports,
                    feedback,
                    background_feedback,
                    completed,
                },
            )));
        }
        self.publish_references();
        self.publish()
    }
    fn shutdown(&mut self, _: Timestamp) -> io::Result<()> {
        self.controller
            .shutdown()
            .map_err(|_| io::Error::other("submission shutdown persistence unavailable"))
    }
}
fn feedback_for(error: SubmissionControlError) -> SubmissionFeedback {
    match error {
        SubmissionControlError::Release(tallyline_workflows::api::ManualReleaseError::Blocked(
            blocks,
        ))
        | SubmissionControlError::Dispatch(
            tallyline_workflows::api::ManualDispatchError::Blocked(blocks),
        ) => SubmissionFeedback::Blocked(blocks),
        SubmissionControlError::Conflict
        | SubmissionControlError::Release(tallyline_workflows::api::ManualReleaseError::Conflict)
        | SubmissionControlError::Dispatch(
            tallyline_workflows::api::ManualDispatchError::Conflict,
        )
        | SubmissionControlError::Repository(
            tallyline_submission::api::model::SubmissionRepositoryError::Conflict,
        ) => SubmissionFeedback::Changed,
        SubmissionControlError::Refused(reason) => SubmissionFeedback::Refused(reason),
        SubmissionControlError::AttachmentUnavailable => SubmissionFeedback::AttachmentUnavailable,
        SubmissionControlError::Dispatch(tallyline_workflows::api::ManualDispatchError::Held(
            reason,
        )) => SubmissionFeedback::Held(reason),
        SubmissionControlError::Dispatch(
            tallyline_workflows::api::ManualDispatchError::Returned(reason),
        ) => SubmissionFeedback::Returned(reason),
        SubmissionControlError::Release(
            tallyline_workflows::api::ManualReleaseError::Invalid
            | tallyline_workflows::api::ManualReleaseError::Unavailable,
        )
        | SubmissionControlError::Dispatch(
            tallyline_workflows::api::ManualDispatchError::Invalid
            | tallyline_workflows::api::ManualDispatchError::Unavailable,
        )
        | SubmissionControlError::Repository(
            tallyline_submission::api::model::SubmissionRepositoryError::Invalid
            | tallyline_submission::api::model::SubmissionRepositoryError::Unavailable,
        )
        | SubmissionControlError::Unavailable
        | SubmissionControlError::Invalid => SubmissionFeedback::Unavailable,
    }
}
