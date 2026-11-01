// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Read-only credential validation contracts against a local HTTP server.

use crate::api::{ClientIdentity, ReaderSettings, UexReader};
use serde_json::json;
use tallyline_submission::api::CredentialValidator;
use tallyline_submission::api::model::{CredentialFailure, CredentialValues, ValidatedAccount};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn r_api_4_r_api_7__valid_account_and_empty_filtered_reports_establish_the_username() {
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/2.0/user/"))
        .and(header("secret-key", "dummy-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok", "data": {
            "username":"dummy-runner", "is_datarunner":1, "is_datarunner_banned":0, "is_staff":0,
            "email":"discarded@example.invalid", "discord":"discarded-dummy-discord"
        }}))).expect(1).mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/2.0/data_info/"))
        .and(query_param("username", "dummy-runner"))
        .and(query_param("limit", "1"))
        .and(header("secret-key", "dummy-key"))
        .and(header("authorization", "Bearer dummy-token"))
        .and(header("x-client-version", "0.0.0"))
        .and(header("user-agent", "tallyline-test/0.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok", "data":[]})))
        .expect(1)
        .mount(&server)
        .await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "0.0.0"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let values =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    assert_eq!(
        reader.validate(&values).await,
        Ok(ValidatedAccount::new("dummy-runner".to_owned()))
    );
    let requests = server.received_requests().await.unwrap();
    assert!(
        !requests
            .first()
            .unwrap()
            .headers
            .contains_key("authorization")
    );
}

#[tokio::test]
async fn r_api_4_r_nf_4__remote_error_text_never_crosses_the_credential_boundary() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2.0/user/"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "status":"invalid_secret_key", "message":"dummy-error-secret-sentinel",
            "data":{"secret":"dummy-error-secret-sentinel"}
        })))
        .mount(&server)
        .await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "0.0.0"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let values =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let outcome = reader.validate(&values).await;
    assert_eq!(outcome, Err(CredentialFailure::InvalidKey));
    assert!(!format!("{outcome:?}").contains("dummy-error-secret-sentinel"));
}

#[tokio::test]
async fn r_api_7__missing_client_version_is_a_client_failure_and_never_a_bad_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2.0/user/"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "status":"missing_client_version", "message":"dummy-private-server-message", "data":null
        })))
        .mount(&server)
        .await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "0.0.0"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let values =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    assert_eq!(
        reader.validate(&values).await,
        Err(CredentialFailure::ClientVersionMissing)
    );
}
