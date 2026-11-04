// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Credential validation requests and save-before-publish application operations.

pub mod model;

use model::SubmissionJob;
use model::VerifiedUexReport;
use model::{
    AccountState, CredentialCandidate, CredentialFailure, CredentialValues, Persistence,
    SavedCredentials, StorageFailure, ValidatedAccount,
};
use std::future::Future;
use tallyline_shared_kernel::api::model::Timestamp;

/// Performs read-only account and selected-token validation.
pub trait CredentialValidator {
    /// Returns only the account identity after both checks succeed.
    fn validate(
        &self,
        values: &CredentialValues,
    ) -> impl std::future::Future<Output = Result<ValidatedAccount, CredentialFailure>> + Send;
}

/// Persists one complete credential record atomically.
pub trait SecretStore {
    /// Reads the required account record without revealing it to a view.
    fn load(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<SavedCredentials>, StorageFailure>> + Send;
    /// Replaces one complete record; an uncertain mutation requires load reconciliation before retry.
    fn save(
        &mut self,
        credentials: &SavedCredentials,
    ) -> impl std::future::Future<Output = Result<(), StorageFailure>> + Send;
    /// Saves revalidation metadata without unlocking or prompting.
    fn save_passive(
        &mut self,
        credentials: &SavedCredentials,
    ) -> impl std::future::Future<Output = Result<(), StorageFailure>> + Send;
    /// Removes only the application's credential record.
    fn remove(&mut self) -> impl std::future::Future<Output = Result<(), StorageFailure>> + Send;
}

/// Owns a candidate until validation and persistence complete.
pub struct ValidationRequest {
    identity: u64,
    candidate: CredentialCandidate,
    revalidation: bool,
}

impl ValidationRequest {
    /// Runs both read-only checks without publishing or saving the candidate.
    ///
    /// # Errors
    /// Returns a missing-token, input or closed remote-validation failure.
    pub async fn validate(
        &self,
        validator: &impl CredentialValidator,
    ) -> Result<ValidatedAccount, CredentialFailure> {
        self.validate_with_release_token(validator, None).await
    }

    /// Uses the own-token override or borrows release installation data for validation.
    ///
    /// # Errors
    /// Returns a missing-token, input or closed remote-validation failure.
    pub async fn validate_with_release_token(
        &self,
        validator: &impl CredentialValidator,
        release_token: Option<&str>,
    ) -> Result<ValidatedAccount, CredentialFailure> {
        let token = self
            .candidate
            .values
            .own_token()
            .or(release_token)
            .ok_or(CredentialFailure::MissingToken)?;
        let selected = CredentialValues::new(
            self.candidate.values.secret_key().to_owned(),
            Some(token.to_owned()),
        )?;
        validator.validate(&selected).await
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum ReleaseEligibility {
    Remembered,
    #[default]
    Closed,
}

/// Retains the last accepted credential identity independently of a new candidate.
#[derive(Default)]
pub struct CredentialApplication {
    generation: u64,
    accepted: Option<SavedCredentials>,
    state: AccountState,
    persistence: Option<Persistence>,
    send_authorised: bool,
    release_eligibility: ReleaseEligibility,
}

impl CredentialApplication {
    /// Restores the last successful account identity from secure storage.
    ///
    /// # Errors
    /// Returns a safe storage failure without exposing a stored record.
    pub async fn load(store: &mut impl SecretStore) -> Result<Self, StorageFailure> {
        let accepted = store.load().await?;
        let state = accepted
            .as_ref()
            .map_or(AccountState::Unconfigured, |saved| {
                account_state(saved, Persistence::Durable)
            });
        let persistence = accepted.as_ref().map(|_| Persistence::Durable);
        Ok(Self {
            generation: 0,
            accepted,
            state,
            persistence,
            send_authorised: false,
            release_eligibility: ReleaseEligibility::Remembered,
        })
    }

    /// Returns safe account state without exposing saved credential values.
    #[must_use]
    pub const fn state(&self) -> &AccountState {
        &self.state
    }

    /// Borrows the last validated account eligible for release without granting send authority.
    #[must_use]
    pub fn release_account(&self) -> Option<&str> {
        if self.release_eligibility != ReleaseEligibility::Remembered {
            return None;
        }
        match &self.state {
            AccountState::Validated { username, .. } => Some(username),
            AccountState::Unconfigured | AccountState::Rejected(_) => None,
        }
    }

    /// Borrows accepted values only after successful validation in this session.
    #[must_use]
    pub fn credentials_for(&self, account: &str) -> Option<&CredentialValues> {
        if !self.send_authorised {
            return None;
        }
        match &self.state {
            AccountState::Validated { username, .. } if username == account => {
                let saved = self.accepted.as_ref()?;
                if saved.account.username == account && saved.refusal.is_none() {
                    Some(&saved.values)
                } else {
                    None
                }
            }
            AccountState::Unconfigured
            | AccountState::Rejected(_)
            | AccountState::Validated { .. } => None,
        }
    }

    /// Starts a candidate, closes release and sending, and supersedes earlier validation requests.
    /// The accepted identity remains visible until a current successful candidate replaces it.
    pub fn begin(&mut self, candidate: CredentialCandidate) -> ValidationRequest {
        self.send_authorised = false;
        self.release_eligibility = ReleaseEligibility::Closed;
        self.generation = self.generation.wrapping_add(1);
        ValidationRequest {
            identity: self.generation,
            candidate,
            revalidation: false,
        }
    }

    /// Rechecks the accepted identity at startup or before its first send.
    /// Retains existing release eligibility without reopening a replacement candidate.
    pub fn begin_revalidation(&mut self) -> Option<ValidationRequest> {
        let saved = self.accepted.as_ref()?;
        let persistence = self.persistence?;
        let candidate = CredentialCandidate::new(saved.values.clone(), persistence);
        let eligibility = self.release_eligibility;
        let mut request = self.begin(candidate);
        self.release_eligibility = eligibility;
        request.revalidation = true;
        Some(request)
    }

    /// Saves a successful current candidate before publishing its username.
    ///
    /// # Errors
    /// Returns validation, supersession or safe storage failure and retains the accepted record.
    pub async fn accept(
        &mut self,
        request: ValidationRequest,
        result: Result<ValidatedAccount, CredentialFailure>,
        now: Timestamp,
        store: &mut impl SecretStore,
    ) -> Result<(), CredentialFailure> {
        if request.identity != self.generation {
            return Err(CredentialFailure::Superseded);
        }
        let account = match result {
            Ok(account) => account,
            Err(failure) => {
                if request.revalidation && known_refusal(failure) {
                    self.state = AccountState::Rejected(failure);
                    if let Some(saved) = &mut self.accepted {
                        saved.refusal = Some(failure);
                        if request.candidate.persistence == Persistence::Durable {
                            store
                                .save_passive(saved)
                                .await
                                .map_err(CredentialFailure::Storage)?;
                        }
                    }
                }
                return Err(failure);
            }
        };
        let saved = SavedCredentials {
            values: request.candidate.values,
            account,
            validated_at: now,
            refusal: None,
        };
        if request.candidate.persistence == Persistence::Durable {
            let result = if request.revalidation {
                store.save_passive(&saved).await
            } else {
                store.save(&saved).await
            };
            result.map_err(CredentialFailure::Storage)?;
        }
        self.state = account_state(&saved, request.candidate.persistence);
        self.persistence = Some(request.candidate.persistence);
        self.accepted = Some(saved);
        self.send_authorised = true;
        self.release_eligibility = ReleaseEligibility::Remembered;
        Ok(())
    }
}

fn account_state(saved: &SavedCredentials, persistence: Persistence) -> AccountState {
    if let Some(failure) = saved.refusal {
        return AccountState::Rejected(failure);
    }
    AccountState::Validated {
        username: saved.account.username.clone(),
        validated_at: saved.validated_at,
        persistence,
    }
}

fn known_refusal(failure: CredentialFailure) -> bool {
    match failure {
        CredentialFailure::InvalidKey
        | CredentialFailure::NotAllowed
        | CredentialFailure::InvalidToken
        | CredentialFailure::UpdateRequired => true,
        CredentialFailure::InvalidInput
        | CredentialFailure::MissingToken
        | CredentialFailure::Unavailable
        | CredentialFailure::ClientVersionMissing
        | CredentialFailure::Superseded
        | CredentialFailure::Storage(_) => false,
    }
}

pub use crate::screenshot_policy::{ScreenshotPolicyApplication, ScreenshotPolicyStore};

/// Sends one prepared released snapshot through the application's shared client.
pub trait SubmissionGateway {
    /// An adapter-owned body prepared outside asynchronous polling.
    type Prepared: Send;
    /// Serialises and hashes one immutable request on a bounded application worker.
    ///
    /// # Errors
    /// Refuses unsupported payload values without dispatching an attempt.
    fn prepare(
        &self,
        request: &model::SubmissionRequest,
    ) -> Result<(Self::Prepared, model::PayloadHash), model::PreparationFailure>;
    /// Performs exactly one prepared attempt without retrying uncertain writes.
    fn submit(
        &self,
        prepared: Self::Prepared,
        credentials: &model::CredentialValues,
        release_token: Option<&str>,
    ) -> impl std::future::Future<Output = model::SubmissionResult> + Send;
}
pub use crate::budget::SubmissionBudget;
pub use crate::preflight::SubmissionPreflight;

/// Supplies both wall time and one stable monotonic origin for submission metadata.
pub trait SubmissionClock: tallyline_shared_kernel::api::Clock + Send + Sync {
    /// Returns elapsed monotonic time independently of wall-clock adjustments.
    fn monotonic(&self) -> std::time::Duration;
}
pub use crate::application::{SubmissionApplication, SubmissionJobRepository};
pub use crate::schedule::SubmissionSchedule;

/// Performs bounded identity reads only after production acceptance is durable.
pub trait SubmissionReceiptVerifier {
    /// Verifies every unmapped identity against its exact released row and attempted wire date.
    /// A failed or incomplete lookup returns absence without changing known acceptance.
    fn verify_receipts(
        &self,
        job: &model::SubmissionJob,
        credentials: &model::CredentialValues,
        release_token: Option<&str>,
    ) -> impl std::future::Future<Output = Option<Vec<model::VerifiedUexReport>>> + Send;
}

pub use crate::duplicate_evidence::SubmissionEvidence;
pub use crate::observation_order::ObservationOrder;
pub use crate::service_fence::SubmissionServiceFenceRepository;

/// Reads positive identities for an uncertain production attempt without inferring non-receipt.
pub trait SubmissionRowReceiptVerifier {
    /// Returns positively matched rows; empty or unavailable windows establish no negative proof.
    fn verify_row_receipts(
        &self,
        job: &SubmissionJob,
        credentials: &CredentialValues,
        release_token: Option<&str>,
    ) -> impl Future<Output = Option<Vec<VerifiedUexReport>>> + Send;
}
