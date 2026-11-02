// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Credential worker contracts without native providers or remote services.

use std::sync::mpsc;
use std::time::Duration;
use tallyline_shared_kernel::api::model::Timestamp;
use tallyline_submission::api::model::{
    AccountState, CredentialCandidate, CredentialFailure, CredentialValues, Persistence,
    SavedCredentials, StorageFailure, ValidatedAccount,
};
use tallyline_submission::api::{CredentialValidator, SecretStore};
use tallyline_ui::api::model::{BackgroundEvent, CredentialCommand};

struct Validator;
impl CredentialValidator for Validator {
    async fn validate(&self, _: &CredentialValues) -> Result<ValidatedAccount, CredentialFailure> {
        Ok(ValidatedAccount::new("dummy-runner".to_owned()))
    }
}
struct Store;
impl SecretStore for Store {
    async fn load(&mut self) -> Result<Option<SavedCredentials>, StorageFailure> {
        Ok(None)
    }
    async fn save(&mut self, _: &SavedCredentials) -> Result<(), StorageFailure> {
        Err(StorageFailure::Locked)
    }
    async fn save_passive(&mut self, saved: &SavedCredentials) -> Result<(), StorageFailure> {
        self.save(saved).await
    }
    async fn remove(&mut self) -> Result<(), StorageFailure> {
        Ok(())
    }
}
struct Clock;
impl tallyline_shared_kernel::api::Clock for Clock {
    fn now(&self) -> Timestamp {
        Timestamp::from_seconds(123)
    }
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_4_r_nf_4__failed_durable_save_is_not_published_and_session_choice_is_explicit() {
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(4);
    let worker = tallyline_app::api::CredentialWorker::start(
        || async { Ok(Store) },
        Validator,
        Clock,
        None,
        incoming,
        outgoing,
        || (),
    )
    .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::CredentialsUpdated {
            account: AccountState::Unconfigured,
            failure: None,
            ..
        }
    ));
    for persistence in [Persistence::Durable, Persistence::Session] {
        commands
            .send(CredentialCommand::Validate(CredentialCandidate::new(
                CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned()))
                    .unwrap(),
                persistence,
            )))
            .ok()
            .unwrap();
        let event = events.recv_timeout(Duration::from_secs(10)).unwrap();
        match persistence {
            Persistence::Durable => assert!(matches!(
                event,
                BackgroundEvent::CredentialsUpdated {
                    account: AccountState::Unconfigured,
                    failure: Some(CredentialFailure::Storage(StorageFailure::Locked)),
                    ..
                }
            )),
            Persistence::Session => assert!(matches!(
                event,
                BackgroundEvent::CredentialsUpdated {
                    account: AccountState::Validated {
                        persistence: Persistence::Session,
                        ..
                    },
                    failure: None,
                    ..
                }
            )),
        }
    }
    drop(commands);
    worker.finish().unwrap();
}

struct UncertainStore {
    dirty: bool,
}
impl SecretStore for UncertainStore {
    async fn load(&mut self) -> Result<Option<SavedCredentials>, StorageFailure> {
        if self.dirty {
            Err(StorageFailure::Corrupt)
        } else {
            Ok(None)
        }
    }
    async fn save(&mut self, _: &SavedCredentials) -> Result<(), StorageFailure> {
        if self.dirty {
            Ok(())
        } else {
            self.dirty = true;
            Err(StorageFailure::OutcomeUnknown)
        }
    }
    async fn save_passive(&mut self, saved: &SavedCredentials) -> Result<(), StorageFailure> {
        self.save(saved).await
    }
    async fn remove(&mut self) -> Result<(), StorageFailure> {
        Ok(())
    }
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_4__unknown_mutation_requires_successful_load_before_any_further_durable_save() {
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(4);
    let worker = tallyline_app::api::CredentialWorker::start(
        || async { Ok(UncertainStore { dirty: false }) },
        Validator,
        Clock,
        None,
        incoming,
        outgoing,
        || (),
    )
    .unwrap();
    events.recv_timeout(Duration::from_secs(10)).unwrap();
    for expected in [StorageFailure::OutcomeUnknown, StorageFailure::Corrupt] {
        commands
            .send(CredentialCommand::Validate(CredentialCandidate::new(
                CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned()))
                    .unwrap(),
                Persistence::Durable,
            )))
            .ok()
            .unwrap();
        assert!(
            matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), BackgroundEvent::CredentialsUpdated { account: AccountState::Unconfigured, failure: Some(CredentialFailure::Storage(failure)), .. } if failure == expected)
        );
    }
    commands
        .send(CredentialCommand::Validate(CredentialCandidate::new(
            CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap(),
            Persistence::Session,
        )))
        .ok()
        .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::CredentialsUpdated {
            account: AccountState::Validated {
                persistence: Persistence::Session,
                ..
            },
            failure: None,
            ..
        }
    ));
    drop(commands);
    worker.finish().unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_4__fallback_requires_unavailable_primary_and_a_separate_warning_confirmation() {
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(4);
    let worker = tallyline_app::api::CredentialWorker::start(
        || async {
            tallyline_app::api::CredentialStores::new(Err(StorageFailure::Unavailable), Some(Store))
        },
        Validator,
        Clock,
        None,
        incoming,
        outgoing,
        || (),
    )
    .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::CredentialsUpdated {
            fallback_available: true,
            account: AccountState::Unconfigured,
            ..
        }
    ));
    commands
        .send(CredentialCommand::ConfirmFallback)
        .ok()
        .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::CredentialsUpdated {
            fallback_available: false,
            account: AccountState::Unconfigured,
            failure: None,
            ..
        }
    ));
    drop(commands);
    worker.finish().unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_4__confirming_an_empty_fallback_keeps_an_accepted_session_identity() {
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(4);
    let worker = tallyline_app::api::CredentialWorker::start(
        || async {
            tallyline_app::api::CredentialStores::new(Err(StorageFailure::Unavailable), Some(Store))
        },
        Validator,
        Clock,
        None,
        incoming,
        outgoing,
        || (),
    )
    .unwrap();
    events.recv_timeout(Duration::from_secs(10)).unwrap();
    commands
        .send(CredentialCommand::Validate(CredentialCandidate::new(
            CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap(),
            Persistence::Session,
        )))
        .ok()
        .unwrap();
    events.recv_timeout(Duration::from_secs(10)).unwrap();
    commands
        .send(CredentialCommand::ConfirmFallback)
        .ok()
        .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::CredentialsUpdated {
            account: AccountState::Validated {
                persistence: Persistence::Session,
                ..
            },
            failure: None,
            ..
        }
    ));
    drop(commands);
    worker.finish().unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_4__locked_corrupt_and_uncertain_primary_stores_never_offer_or_select_fallback() {
    for failure in [
        StorageFailure::Locked,
        StorageFailure::Corrupt,
        StorageFailure::OutcomeUnknown,
        StorageFailure::WriteFailed,
    ] {
        let (commands, incoming) = mpsc::sync_channel(1);
        let (outgoing, events) = mpsc::sync_channel(4);
        let worker =
            tallyline_app::api::CredentialWorker::start(
                move || async move {
                    tallyline_app::api::CredentialStores::new(Err(failure), Some(Store))
                },
                Validator,
                Clock,
                None,
                incoming,
                outgoing,
                || (),
            )
            .unwrap();
        assert!(
            matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), BackgroundEvent::CredentialsUpdated { fallback_available: false, failure: Some(CredentialFailure::Storage(actual)), .. } if actual == failure)
        );
        commands
            .send(CredentialCommand::ConfirmFallback)
            .ok()
            .unwrap();
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(10)).unwrap(),
            BackgroundEvent::CredentialsUpdated {
                fallback_available: false,
                failure: Some(CredentialFailure::Storage(StorageFailure::Unavailable)),
                ..
            }
        ));
        drop(commands);
        worker.finish().unwrap();
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_4_r_nf_4__installation_data_is_prepared_on_the_credential_worker_before_session_validation()
 {
    let caller = std::thread::current().id();
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(4);
    let worker = tallyline_app::api::CredentialWorker::start(
        move || async move {
            assert_ne!(std::thread::current().id(), caller);
            assert_eq!(std::thread::current().name(), Some("tallyline-credentials"));
            tallyline_app::api::CredentialStores::new(Ok(Store), None).with_release_token(
                tallyline_submission::api::model::ReleaseToken::new(
                    "dummy-installation-token".to_owned(),
                )
                .ok(),
            )
        },
        Validator,
        Clock,
        None,
        incoming,
        outgoing,
        || (),
    )
    .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::CredentialsUpdated {
            available_token: true,
            ..
        }
    ));
    commands
        .send(CredentialCommand::Validate(CredentialCandidate::new(
            CredentialValues::new("dummy-key".to_owned(), None).unwrap(),
            Persistence::Session,
        )))
        .ok()
        .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::CredentialsUpdated {
            account: AccountState::Validated {
                persistence: Persistence::Session,
                ..
            },
            failure: None,
            ..
        }
    ));
    drop(commands);
    worker.finish().unwrap();
}

struct RestoredStore {
    saved: std::sync::Arc<std::sync::Mutex<SavedCredentials>>,
}
impl SecretStore for RestoredStore {
    async fn load(&mut self) -> Result<Option<SavedCredentials>, StorageFailure> {
        self.saved
            .lock()
            .map(|saved| Some(saved.clone()))
            .map_err(|_| StorageFailure::Unavailable)
    }
    async fn save(&mut self, _: &SavedCredentials) -> Result<(), StorageFailure> {
        Err(StorageFailure::WriteFailed)
    }
    async fn save_passive(&mut self, saved: &SavedCredentials) -> Result<(), StorageFailure> {
        *self.saved.lock().map_err(|_| StorageFailure::Unavailable)? = saved.clone();
        Ok(())
    }
    async fn remove(&mut self) -> Result<(), StorageFailure> {
        Err(StorageFailure::WriteFailed)
    }
}
struct ClosedValidator(CredentialFailure);
impl CredentialValidator for ClosedValidator {
    async fn validate(&self, _: &CredentialValues) -> Result<ValidatedAccount, CredentialFailure> {
        Err(self.0)
    }
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_4_r_api_7_r_nf_4_r_ui_16__restored_account_is_revalidated_passively_and_refusal_survives_offline_restart()
 {
    let saved = std::sync::Arc::new(std::sync::Mutex::new(
        SavedCredentials::from_storage(
            CredentialValues::new(
                "dummy-startup-key".to_owned(),
                Some("dummy-startup-token".to_owned()),
            )
            .unwrap(),
            ValidatedAccount::new("dummy-runner".to_owned()),
            Timestamp::from_seconds(10),
            None,
        )
        .unwrap(),
    ));
    for failure in [
        CredentialFailure::InvalidKey,
        CredentialFailure::Unavailable,
    ] {
        let saved = std::sync::Arc::clone(&saved);
        let (commands, incoming) = mpsc::sync_channel(1);
        let (outgoing, events) = mpsc::sync_channel(4);
        let worker = tallyline_app::api::CredentialWorker::start(
            move || async move { Ok(RestoredStore { saved }) },
            ClosedValidator(failure),
            Clock,
            None,
            incoming,
            outgoing,
            || (),
        )
        .unwrap();
        assert!(
            matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), BackgroundEvent::CredentialsUpdated {
            account: AccountState::Rejected(CredentialFailure::InvalidKey), failure: Some(actual), ..
        } if actual == failure)
        );
        drop(commands);
        worker.finish().unwrap();
    }
}
