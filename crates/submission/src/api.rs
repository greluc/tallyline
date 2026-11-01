// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Credential validation requests and save-before-publish application operations.

pub mod model;

use model::{
    AccountState, CredentialCandidate, CredentialFailure, CredentialValues, Persistence,
    SavedCredentials, StorageFailure, ValidatedAccount,
};
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
    /// Replaces the record, preserving the previous value on failure.
    fn save(
        &mut self,
        credentials: &SavedCredentials,
    ) -> impl std::future::Future<Output = Result<(), StorageFailure>> + Send;
    /// Removes only the application's credential record.
    fn remove(&mut self) -> impl std::future::Future<Output = Result<(), StorageFailure>> + Send;
}

/// Owns a candidate until validation and persistence complete.
#[derive(Debug)]
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

/// Retains the last accepted credential identity independently of a new candidate.
#[derive(Default)]
pub struct CredentialApplication {
    generation: u64,
    accepted: Option<SavedCredentials>,
    state: AccountState,
    persistence: Option<Persistence>,
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
        })
    }

    /// Returns safe account state without exposing saved credential values.
    #[must_use]
    pub const fn state(&self) -> &AccountState {
        &self.state
    }

    /// Starts a candidate and supersedes all earlier validation requests.
    pub fn begin(&mut self, candidate: CredentialCandidate) -> ValidationRequest {
        self.generation = self.generation.wrapping_add(1);
        ValidationRequest {
            identity: self.generation,
            candidate,
            revalidation: false,
        }
    }

    /// Rechecks the accepted identity at startup or before its first send.
    pub fn begin_revalidation(&mut self) -> Option<ValidationRequest> {
        let saved = self.accepted.as_ref()?;
        let persistence = self.persistence?;
        let candidate = CredentialCandidate::new(saved.values.clone(), persistence);
        let mut request = self.begin(candidate);
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
                                .save(saved)
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
            store
                .save(&saved)
                .await
                .map_err(CredentialFailure::Storage)?;
        }
        self.state = account_state(&saved, request.candidate.persistence);
        self.persistence = Some(request.candidate.persistence);
        self.accepted = Some(saved);
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
