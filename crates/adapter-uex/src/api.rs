// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! UEX client settings, credentials, request outcomes and the read and dry-run operations.

pub mod model;

use std::fmt;
use tallyline_reference_data::api::model::Terminal;

use crate::transport::Transport;
use crate::wire;
use model::{
    DryRunReport, GameVersions, PriceRow, ReportAccess, StatusLevels, SubmitSuccess, TerminalId,
    UexAccount,
};

const DEFAULT_MAX_BODY_BYTES: usize = 16 * 1024 * 1024;
#[cfg(test)]
const LOOPBACK_HOSTS: [&str; 3] = ["127.0.0.1", "localhost", "[::1]"];

/// The client name and build version sent as `User-Agent` and `X-Client-Version` (R-API-7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientIdentity {
    name: String,
    version: String,
}

impl ClientIdentity {
    /// Creates the identity from the client's name and its build version.
    #[must_use]
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_owned(),
            version: version.to_owned(),
        }
    }

    pub(crate) fn user_agent(&self) -> String {
        format!("{}/{}", self.name, self.version)
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }
}

/// The settings of a [`UexReader`]: base URL, identity and response size cap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReaderSettings {
    pub(crate) base_url: reqwest::Url,
    pub(crate) identity: ClientIdentity,
    pub(crate) max_body_bytes: usize,
}

impl ReaderSettings {
    /// Creates settings for `base_url` with a 16 MiB response cap; requests time out after 30 s.
    ///
    /// The base URL ends with a slash and uses `https` (R-API-3).
    ///
    /// # Errors
    ///
    /// [`SettingsError::InvalidBaseUrl`] when the URL cannot be parsed or does not end with a
    /// slash, [`SettingsError::InsecureScheme`] for any other scheme.
    pub fn new(base_url: &str, identity: ClientIdentity) -> Result<Self, SettingsError> {
        let parsed = reqwest::Url::parse(base_url).map_err(|_| SettingsError::InvalidBaseUrl)?;
        if !parsed.path().ends_with('/') || parsed.query().is_some() {
            return Err(SettingsError::InvalidBaseUrl);
        }
        if parsed.scheme() != "https" && !test_loopback(&parsed) {
            return Err(SettingsError::InsecureScheme);
        }
        Ok(Self {
            base_url: parsed,
            identity,
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        })
    }

    /// Returns these settings with another response body cap in bytes (R-API-5).
    ///
    /// # Errors
    ///
    /// [`SettingsError::ZeroBodyLimit`] when `limit` is zero.
    pub fn with_max_body_bytes(self, limit: usize) -> Result<Self, SettingsError> {
        if limit == 0 {
            return Err(SettingsError::ZeroBodyLimit);
        }
        Ok(Self {
            max_body_bytes: limit,
            ..self
        })
    }
}

#[cfg(test)]
fn test_loopback(url: &reqwest::Url) -> bool {
    url.scheme() == "http"
        && url
            .host_str()
            .is_some_and(|host| LOOPBACK_HOSTS.contains(&host))
}

#[cfg(not(test))]
fn test_loopback(_: &reqwest::Url) -> bool {
    false
}

/// Why [`ReaderSettings`] refused a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsError {
    /// The base URL cannot be parsed, has a query or does not end with a slash.
    InvalidBaseUrl,
    /// The base URL does not use `https`.
    InsecureScheme,
    /// The response body cap is zero.
    ZeroBodyLimit,
}

/// The user's UEX secret key, sent as the `secret-key` header; masked in debug output.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretKey(String);

impl SecretKey {
    /// Wraps the secret key.
    #[must_use]
    pub fn new(key: &str) -> Self {
        Self(key.to_owned())
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretKey(***)")
    }
}

/// The user's own UEX app token, sent as `Authorization: Bearer`; masked in debug output.
#[derive(Clone, PartialEq, Eq)]
pub struct AppToken(String);

impl AppToken {
    /// Wraps the app token.
    #[must_use]
    pub fn new(token: &str) -> Self {
        Self(token.to_owned())
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AppToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AppToken(***)")
    }
}

/// Why building a [`UexReader`] failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildError {
    /// The TLS configuration with the `aws-lc-rs` provider and the platform verifier failed.
    Tls(String),
    /// The HTTP client could not be built.
    Client(String),
}

/// Why a read call failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadError {
    /// UEX answered with an error envelope; `code` is its `status` string.
    Rejected {
        /// The HTTP status of the answer.
        http_status: u16,
        /// The UEX code from the envelope's `status`, such as `invalid_secret_key`.
        code: String,
        /// The envelope's `message`, empty when UEX sent none.
        message: String,
        /// Structured error details such as `data.row` or `data.report`.
        data: Option<RejectedData>,
    },
    /// The answer was not JSON, for example the HTML page of an unknown resource.
    NotJson {
        /// The HTTP status of the answer.
        http_status: u16,
    },
    /// UEX answered with a redirect, which is never followed (R-API-3).
    Redirect {
        /// The HTTP status of the answer.
        http_status: u16,
    },
    /// The answer was JSON but not of the documented shape or out of the documented range.
    Malformed {
        /// What did not match, without any response content.
        detail: String,
    },
    /// The response body exceeded the configured cap (R-API-5).
    BodyTooLarge {
        /// The cap in bytes.
        limit: usize,
    },
    /// The request failed before an answer was read: connection, TLS or timeout.
    Transport(String),
}

/// Sanitised structured fields returned with a rejected UEX request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RejectedData {
    /// Zero-based index of the `prices[]` row that failed validation.
    Row(usize),
    /// Report identity attached to `duplicated_report`.
    Report {
        /// UEX terminal ID.
        id_terminal: u64,
        /// UEX commodity ID.
        id_commodity: u64,
    },
    /// UEX returned structured data outside the shapes currently modelled here.
    Other,
}

/// The UEX API 2.0 reader: GET requests only, no redirects, a capped body size.
#[derive(Debug)]
pub struct UexReader {
    pub(crate) transport: Transport,
}

/// The UEX submission client. Its only write operation is a `data_submit` dry run.
#[derive(Debug)]
pub struct UexSubmitter {
    transport: Transport,
}

impl UexSubmitter {
    /// Builds the submitter with the same TLS and response safeguards as [`UexReader`].
    ///
    /// # Errors
    ///
    /// [`BuildError::Tls`] or [`BuildError::Client`] when the TLS stack or HTTP client fails.
    pub fn new(settings: &ReaderSettings) -> Result<Self, BuildError> {
        Ok(Self {
            transport: Transport::new(settings)?,
        })
    }

    /// Submits a report with `is_production=0` and parses the success response.
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError`] for an HTTP, UEX-envelope, or response-shape failure.
    pub async fn dry_run(
        &self,
        token: &AppToken,
        key: &SecretKey,
        report: &DryRunReport,
    ) -> Result<SubmitSuccess, ReadError> {
        let bearer = format!("Bearer {}", token.expose());
        let data = self
            .transport
            .post(
                "data_submit/",
                report,
                &[("authorization", &bearer), ("secret-key", key.expose())],
            )
            .await?;
        serde_json::from_value(data).map_err(|error| ReadError::Malformed {
            detail: format!("data_submit success data: {error}"),
        })
    }
}

impl UexReader {
    /// Builds the client with the `aws-lc-rs` crypto provider and the platform verifier.
    ///
    /// # Errors
    ///
    /// [`BuildError::Tls`] or [`BuildError::Client`] when the TLS stack or the client fails.
    pub fn new(settings: &ReaderSettings) -> Result<Self, BuildError> {
        let transport = Transport::new(settings)?;
        Ok(Self { transport })
    }

    /// Reads selectable commodity terminals without credentials or LIVE filtering.
    ///
    /// # Errors
    /// Returns a transport, envelope or malformed-reference failure.
    pub async fn terminals(&self) -> Result<Vec<Terminal>, ReadError> {
        let data = self
            .transport
            .get("terminals/", &[("type", "commodity")], &[])
            .await?;
        crate::reference_wire::terminals(data)
    }
    /// Reads `GET /game_versions`: the current LIVE and PTU version.
    ///
    /// # Errors
    ///
    /// A [`ReadError`] for every answer other than a well-formed `ok` envelope.
    pub async fn game_versions(&self) -> Result<GameVersions, ReadError> {
        let data = self.transport.get("game_versions/", &[], &[]).await?;
        wire::game_versions(data)
    }

    /// Reads `GET /commodities_status`: the status levels of each side.
    ///
    /// # Errors
    ///
    /// A [`ReadError`] for every answer other than a well-formed `ok` envelope.
    pub async fn commodity_status_levels(&self) -> Result<StatusLevels, ReadError> {
        let data = self.transport.get("commodities_status/", &[], &[]).await?;
        wire::status_levels(data)
    }

    /// Reads `GET /commodities_prices?id_terminal=…`: the price rows of one terminal.
    ///
    /// # Errors
    ///
    /// A [`ReadError`] for every answer other than a well-formed `ok` envelope.
    pub async fn price_rows(&self, terminal: TerminalId) -> Result<Vec<PriceRow>, ReadError> {
        let terminal = terminal.0.to_string();
        let data = self
            .transport
            .get("commodities_prices/", &[("id_terminal", &terminal)], &[])
            .await?;
        wire::price_rows(data)
    }

    /// Reads `GET /user` with the secret key alone and keeps only the account state.
    ///
    /// The e-mail address and Discord name in the answer are dropped here (O-97).
    ///
    /// # Errors
    ///
    /// A [`ReadError`]; an invalid key is `Rejected` with the code `invalid_secret_key`.
    pub async fn account(&self, key: &SecretKey) -> Result<UexAccount, ReadError> {
        let data = self
            .transport
            .get("user/", &[], &[("secret-key", key.expose())])
            .await?;
        wire::account(data)
    }

    /// Reads `GET /data_info` for `username` with `limit=1` to check the app token and the key.
    ///
    /// # Errors
    ///
    /// A [`ReadError`]; a refused token is `Rejected` with `not_allowed` or `access_denied`.
    pub async fn report_access(
        &self,
        token: &AppToken,
        key: &SecretKey,
        username: &str,
    ) -> Result<ReportAccess, ReadError> {
        let bearer = format!("Bearer {}", token.expose());
        let data = self
            .transport
            .get(
                "data_info/",
                &[("username", username), ("limit", "1")],
                &[("authorization", &bearer), ("secret-key", key.expose())],
            )
            .await?;
        wire::report_access(&data)
    }
}
