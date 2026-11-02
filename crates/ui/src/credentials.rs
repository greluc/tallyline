// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Masked credential input and safe account feedback.

use tallyline_submission::api::model::{
    AccountState, CredentialCandidate, CredentialFailure, CredentialValues, Persistence,
};

/// A secret-bearing request without diagnostic or serialisation support.
pub enum CredentialCommand {
    /// Confirms the displayed Linux plaintext fallback warning.
    ConfirmFallback,
    /// Validate and explicitly accept the selected lifetime.
    Validate(CredentialCandidate),
}

/// Short-lived input and safe credential worker results.
#[derive(Default)]
pub struct CredentialSetup {
    key: String,
    token: String,
    own_token: bool,
    persistence: Option<Persistence>,
    phase: SetupPhase,
    available_token: bool,
    fallback_available: bool,
    account: AccountState,
    failure: Option<CredentialFailure>,
    pending: Option<CredentialCommand>,
}
impl CredentialSetup {
    /// Returns the accepted account independently of current input.
    #[must_use]
    pub const fn account(&self) -> &AccountState {
        &self.account
    }
    /// Returns the closed actionable failure.
    #[must_use]
    pub const fn failure(&self) -> Option<CredentialFailure> {
        self.failure
    }
    /// Returns whether validation is pending.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.phase == SetupPhase::Checking
    }
    /// Sets only unsaved masked input after a user edit.
    pub fn set_input(&mut self, key: String, token: Option<String>, persistence: Persistence) {
        self.key = key;
        self.own_token = token.is_some();
        self.token = token.unwrap_or_default();
        self.persistence = Some(persistence);
    }
    /// Queues one candidate and clears the view's input copies.
    pub fn validate(&mut self) {
        if self.phase != SetupPhase::Ready {
            return;
        }
        let token = if self.own_token || !self.available_token {
            Some(std::mem::take(&mut self.token))
        } else {
            self.token.clear();
            None
        };
        match CredentialValues::new(std::mem::take(&mut self.key), token) {
            Ok(values) => {
                self.pending = Some(CredentialCommand::Validate(CredentialCandidate::new(
                    values,
                    self.persistence.unwrap_or(Persistence::Durable),
                )));
                self.phase = SetupPhase::Checking;
                self.failure = None;
            }
            Err(failure) => self.failure = Some(failure),
        }
    }
    /// Drains the bounded command for the app-owned worker.
    pub fn take_command(&mut self) -> Option<CredentialCommand> {
        self.pending.take()
    }
    pub(crate) fn updated(
        &mut self,
        account: AccountState,
        failure: Option<CredentialFailure>,
        available_token: bool,
        fallback_available: bool,
    ) {
        self.account = account;
        self.failure = failure;
        self.available_token = available_token;
        self.fallback_available = fallback_available;
        self.phase = SetupPhase::Ready;
    }
    pub(crate) fn failed_delivery(&mut self) {
        self.failure = Some(CredentialFailure::Unavailable);
        self.phase = SetupPhase::Ready;
    }
}

pub(crate) fn show(
    ui: &mut egui::Ui,
    setup: &mut CredentialSetup,
    catalogue: &crate::api::Catalogue,
) {
    ui.heading(catalogue.text("credentials.title"));
    ui.label(catalogue.text("credentials.local_use"));
    match &setup.account {
        AccountState::Unconfigured => {
            ui.label(catalogue.text("credentials.unconfigured"));
        }
        AccountState::Rejected(failure) => {
            ui.label(catalogue.text(failure_key(*failure)));
        }
        AccountState::Validated {
            username,
            persistence,
            ..
        } => {
            ui.label(format!(
                "{}: {username}",
                catalogue.text("credentials.validated")
            ));
            ui.label(catalogue.text(match persistence {
                Persistence::Durable => "credentials.durable",
                Persistence::Session => "credentials.session",
            }));
        }
    }
    if let Some(failure) = setup.failure {
        ui.label(catalogue.text(failure_key(failure)));
    }
    if setup.fallback_available {
        ui.label(catalogue.text("credentials.fallback_warning"));
        if ui
            .add_enabled(
                !setup.busy(),
                egui::Button::new(catalogue.text("credentials.fallback_confirm")),
            )
            .clicked()
        {
            setup.pending = Some(CredentialCommand::ConfirmFallback);
            setup.phase = SetupPhase::Checking;
        }
    }
    ui.add_enabled_ui(setup.phase == SetupPhase::Ready, |ui| {
        ui.label(catalogue.text("credentials.key"));
        ui.add(
            egui::TextEdit::singleline(&mut setup.key)
                .password(true)
                .char_limit(1024)
                .hint_text(catalogue.text("credentials.key")),
        );
        if setup.available_token {
            ui.checkbox(&mut setup.own_token, catalogue.text("credentials.expert"));
        } else {
            ui.label(catalogue.text("credentials.token_missing"));
        }
        if setup.own_token || !setup.available_token {
            ui.label(catalogue.text("credentials.token"));
            ui.add(
                egui::TextEdit::singleline(&mut setup.token)
                    .password(true)
                    .char_limit(1024)
                    .hint_text(catalogue.text("credentials.token")),
            );
        }
        let mut session = setup.persistence == Some(Persistence::Session);
        if ui
            .checkbox(&mut session, catalogue.text("credentials.session_choice"))
            .changed()
        {
            setup.persistence = Some(if session {
                Persistence::Session
            } else {
                Persistence::Durable
            });
        }
        if ui.button(catalogue.text("credentials.validate")).clicked() {
            setup.validate();
        }
    });
    if setup.busy() {
        ui.label(catalogue.text("credentials.checking"));
    }
}
fn failure_key(failure: CredentialFailure) -> &'static str {
    use tallyline_submission::api::model::StorageFailure;
    match failure {
        CredentialFailure::InvalidInput => "credentials.invalid_input",
        CredentialFailure::InvalidKey => "credentials.invalid_key",
        CredentialFailure::NotAllowed => "credentials.not_allowed",
        CredentialFailure::InvalidToken => "credentials.invalid_token",
        CredentialFailure::MissingToken => "credentials.token_missing",
        CredentialFailure::UpdateRequired | CredentialFailure::ClientVersionMissing => {
            "credentials.update"
        }
        CredentialFailure::Unavailable | CredentialFailure::Superseded => "credentials.unavailable",
        CredentialFailure::Storage(StorageFailure::Unavailable) => {
            "credentials.storage_unavailable"
        }
        CredentialFailure::Storage(StorageFailure::Locked) => "credentials.locked",
        CredentialFailure::Storage(StorageFailure::Corrupt) => "credentials.corrupt",
        CredentialFailure::Storage(StorageFailure::OutcomeUnknown) => "credentials.uncertain",
        CredentialFailure::Storage(StorageFailure::WriteFailed) => "credentials.save_failed",
    }
}

#[derive(Default, PartialEq, Eq)]
enum SetupPhase {
    #[default]
    Starting,
    Ready,
    Checking,
}
