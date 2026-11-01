// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The HTTP transport: one `reqwest` client with the explicit TLS stack and capped response bodies.

use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use rustls::ClientConfig;
use rustls::crypto::aws_lc_rs;
use rustls_platform_verifier::BuilderVerifierExt;
use serde::Serialize;
use serde_json::Value;

use crate::api::{BuildError, ReadError, ReaderSettings};
use crate::envelope;

const CLIENT_VERSION_HEADER: &str = "x-client-version";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// The shared HTTP client of the adapter and the base URL its requests are built against.
#[derive(Debug)]
pub(crate) struct Transport {
    client: reqwest::Client,
    base_url: reqwest::Url,
    max_body_bytes: usize,
}

impl Transport {
    /// Builds the client: `aws-lc-rs` provider, platform verifier, no redirects, fixed headers.
    pub(crate) fn new(settings: &ReaderSettings) -> Result<Self, BuildError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, header_value(&settings.identity.user_agent())?);
        headers.insert(
            HeaderName::from_static(CLIENT_VERSION_HEADER),
            header_value(settings.identity.version())?,
        );
        let client = reqwest::Client::builder()
            .tls_backend_preconfigured(tls_config()?)
            .redirect(reqwest::redirect::Policy::none())
            .default_headers(headers)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(client_error)?;
        Ok(Self {
            client,
            base_url: settings.base_url.clone(),
            max_body_bytes: settings.max_body_bytes,
        })
    }

    /// Sends `GET <base><resource>` and returns the `data` of an `ok` envelope.
    ///
    /// The values of `headers` are marked sensitive.
    pub(crate) async fn get(
        &self,
        resource: &str,
        query: &[(&str, &str)],
        headers: &[(&str, &str)],
    ) -> Result<Value, ReadError> {
        let mut url = self
            .base_url
            .join(resource)
            .map_err(|error| ReadError::Transport(format!("resource path: {error}")))?;
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        let mut request = self.client.get(url);
        for (name, value) in headers {
            let name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|error| ReadError::Transport(error.to_string()))?;
            let mut value = HeaderValue::from_str(value)
                .map_err(|_| ReadError::Transport(format!("header {name} has an invalid value")))?;
            value.set_sensitive(true);
            request = request.header(name, value);
        }
        let response = request.send().await.map_err(transport_error)?;
        self.read_response(response).await
    }

    /// Sends a JSON `POST` and returns the `data` of its response envelope.
    pub(crate) async fn post<T: Serialize>(
        &self,
        resource: &str,
        payload: &T,
        headers: &[(&str, &str)],
    ) -> Result<Value, ReadError> {
        let url = self
            .base_url
            .join(resource)
            .map_err(|error| ReadError::Transport(format!("resource path: {error}")))?;
        let body = serde_json::to_vec(payload).map_err(|error| ReadError::Malformed {
            detail: format!("could not serialise data_submit payload: {error}"),
        })?;
        let mut request = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .body(body);
        for (name, value) in headers {
            let name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|error| ReadError::Transport(error.to_string()))?;
            let mut value = HeaderValue::from_str(value)
                .map_err(|_| ReadError::Transport(format!("header {name} has an invalid value")))?;
            value.set_sensitive(true);
            request = request.header(name, value);
        }
        let response = request.send().await.map_err(transport_error)?;
        self.read_response(response).await
    }

    async fn read_response(&self, mut response: reqwest::Response) -> Result<Value, ReadError> {
        let http_status = response.status().as_u16();
        if response.status().is_redirection() {
            return Err(ReadError::Redirect { http_status });
        }
        let limit = self.max_body_bytes;
        if response
            .content_length()
            .is_some_and(|length| usize::try_from(length).map_or(true, |length| length > limit))
        {
            return Err(ReadError::BodyTooLarge { limit });
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
            if body.len().saturating_add(chunk.len()) > limit {
                return Err(ReadError::BodyTooLarge { limit });
            }
            body.extend_from_slice(&chunk);
        }
        envelope::data(http_status, &body)
    }
}

fn header_value(value: &str) -> Result<HeaderValue, BuildError> {
    HeaderValue::from_str(value).map_err(client_error)
}

fn client_error(error: impl ToString) -> BuildError {
    BuildError::Client(error.to_string())
}

fn transport_error(error: reqwest::Error) -> ReadError {
    ReadError::Transport(error.without_url().to_string())
}

fn tls_config() -> Result<ClientConfig, BuildError> {
    let provider = Arc::new(aws_lc_rs::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .and_then(BuilderVerifierExt::with_platform_verifier)
        .map_err(|error| BuildError::Tls(error.to_string()))?
        .with_no_client_auth();
    Ok(config)
}
