// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Credential values and closed account outcomes.

use tallyline_shared_kernel::api::model::Timestamp;

const MAX_CREDENTIAL_BYTES: usize = 1024;

/// Credentials retained only at the secure storage and validation boundaries.
#[derive(Clone, PartialEq, Eq)]
pub struct CredentialValues {
    key: String,
    own_token: Option<String>,
}

impl CredentialValues {
    /// Accepts bounded non-empty printable ASCII credentials.
    ///
    /// # Errors
    /// Returns an input failure without retaining malformed credentials.
    pub fn new(key: String, own_token: Option<String>) -> Result<Self, CredentialFailure> {
        if !valid_secret(&key)
            || own_token
                .as_deref()
                .is_some_and(|token| !valid_secret(token))
        {
            return Err(CredentialFailure::InvalidInput);
        }
        Ok(Self { key, own_token })
    }

    /// Borrows the key exclusively for a credential boundary operation.
    #[must_use]
    pub fn secret_key(&self) -> &str {
        &self.key
    }

    /// Borrows an own token exclusively for a credential boundary operation.
    #[must_use]
    pub fn own_token(&self) -> Option<&str> {
        self.own_token.as_deref()
    }
}

fn valid_secret(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CREDENTIAL_BYTES
        && value.bytes().all(|byte| byte.is_ascii_graphic())
}

/// The explicitly requested storage lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Persistence {
    /// Retain credentials securely across restarts.
    Durable,
    /// Retain credentials in memory until application exit.
    Session,
}

/// A proposed credential replacement which has not yet been validated.
pub struct CredentialCandidate {
    pub(crate) values: CredentialValues,
    pub(crate) persistence: Persistence,
}

impl CredentialCandidate {
    /// Creates a candidate with an explicit storage lifetime.
    #[must_use]
    pub const fn new(values: CredentialValues, persistence: Persistence) -> Self {
        Self {
            values,
            persistence,
        }
    }
}

/// A username established by the account and token validation calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedAccount {
    pub(crate) username: String,
}

impl ValidatedAccount {
    /// Creates the reduced account result after validating DataRunner standing.
    #[must_use]
    pub const fn new(username: String) -> Self {
        Self { username }
    }
}

/// A versioned secure-store record containing only required account data.
#[derive(Clone, PartialEq, Eq)]
pub struct SavedCredentials {
    pub(crate) values: CredentialValues,
    pub(crate) account: ValidatedAccount,
    pub(crate) validated_at: Timestamp,
    pub(crate) refusal: Option<CredentialFailure>,
}

impl SavedCredentials {
    /// Borrows credential values only for secure-store encoding.
    #[must_use]
    pub const fn values(&self) -> &CredentialValues {
        &self.values
    }
}

/// Safe account information that can be shown without revealing credentials.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AccountState {
    /// No successfully validated credential identity is available.
    #[default]
    Unconfigured,
    /// The current identity is known to be refused and cannot authorise release.
    Rejected(CredentialFailure),
    /// The current identity passed both read-only checks.
    Validated {
        /// The validated UEX account identity.
        username: String,
        /// The last successful validation time.
        validated_at: Timestamp,
        /// The accepted credential lifetime.
        persistence: Persistence,
    },
}

/// A closed validation outcome containing no remote message or secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialFailure {
    /// Credential input is empty, excessive or unsuitable for an HTTP header.
    InvalidInput,
    /// The user's key was refused.
    InvalidKey,
    /// The account may not submit DataRunner reports.
    NotAllowed,
    /// The selected app token was refused.
    InvalidToken,
    /// No release or own app token is available.
    MissingToken,
    /// This client must be updated before validation can complete.
    UpdateRequired,
    /// UEX reported that the required client-version header was absent.
    ClientVersionMissing,
    /// UEX could not be reached or returned an unusable response.
    Unavailable,
    /// A newer request superseded this result.
    Superseded,
    /// The requested secure persistence failed.
    Storage(StorageFailure),
}

/// A safe secure-store outcome without dependency error payloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageFailure {
    /// No supported secure store is available.
    Unavailable,
    /// The store is locked or access was declined.
    Locked,
    /// The stored record is malformed or ambiguous.
    Corrupt,
    /// The requested write could not be committed.
    WriteFailed,
}
