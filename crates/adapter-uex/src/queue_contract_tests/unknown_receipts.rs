// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Positive-only receipt association through isolated HTTP.

use crate::api::{ClientIdentity, ReaderSettings, UexReader};
use serde_json::Value;
use serde_json::json;
use tallyline_shared_kernel::api::model::{SubmissionMode, Timestamp};
use tallyline_submission::api::SubmissionUnknownReceiptVerifier;
use tallyline_submission::api::model::VerifiedUexReport;
use tallyline_submission::api::model::{
    CredentialValues, JobPhase, SubmissionRequest, SubmissionResult,
};
use wiremock::matchers;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_9__unknown_receipt_read_uses_exact_attempted_date_and_never_posts() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/2.0/data_info/"))
        .and(matchers::query_param("type", "commodity"))
        .and(matchers::query_param("id_terminal", "89"))
        .and(matchers::query_param("username", "dummy-a"))
        .and(matchers::query_param("limit", "100"))
        .and(matchers::header("authorization", "Bearer dummy-token"))
        .and(matchers::header("secret-key", "dummy-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status":"ok","data":[
                {"id":81,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":100},
                {"id":82,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":95}
            ]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let base = super::released_request().unwrap();
    let mut header = base.header().clone();
    header.mode = SubmissionMode::Production;
    let request = SubmissionRequest::new(header, base.rows().to_vec()).unwrap();
    let job = super::accepted_job(
        &request,
        SubmissionResult::Unknown,
        Timestamp::from_seconds(95),
    )
    .unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let receipts = reader
        .verify_unknown_receipts(&job, &credentials, None)
        .await;
    assert!(
        receipts.is_some(),
        "one positive exact-date lookup must be retained: {receipts:?}"
    );
    let receipts = receipts.unwrap();
    assert_eq!(
        receipts
            .iter()
            .map(|receipt| (
                receipt.row().0,
                receipt.id(),
                receipt.date_added().seconds()
            ))
            .collect::<Vec<_>>(),
        vec![(1, 82, 95)]
    );
    assert_eq!(job.phase(), JobPhase::OutcomeUnknown);
    assert_eq!(job.result(), Some(&SubmissionResult::Unknown));
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method.as_str(), "GET");
    assert!(requests[0].url.query_pairs().all(|(key, _)| key != "id"));
    server.verify().await;
}

async fn read_window(data: Value) -> Option<Vec<VerifiedUexReport>> {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/2.0/data_info/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok","data":data})))
        .expect(1)
        .mount(&server)
        .await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let base = super::released_request().unwrap();
    let mut header = base.header().clone();
    header.mode = SubmissionMode::Production;
    let request = SubmissionRequest::new(header, base.rows().to_vec()).unwrap();
    let job = super::accepted_job(
        &request,
        SubmissionResult::Unknown,
        Timestamp::from_seconds(95),
    )
    .unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let result = reader
        .verify_unknown_receipts(&job, &credentials, None)
        .await;
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    server.verify().await;
    result
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_9__ambiguous_unknown_receipt_window_never_selects_first_matching_identity() {
    let duplicated =
        json!({"id":82,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":95});
    assert!(
        read_window(json!([duplicated.clone(), duplicated]))
            .await
            .is_none(),
        "repeated matching records are ambiguous, not two accepted rows"
    );
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_9__malformed_unknown_window_cannot_supply_receipt_authority() {
    let data = json!([
        {"id":82,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":95},
        {"id":83,"type":"commodity","id_terminal":89,"id_commodity":8}
    ]);
    assert!(
        read_window(data).await.is_none(),
        "malformed windows remain unavailable even beside a matching record"
    );
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_9__empty_full_and_oversized_unknown_windows_never_prove_non_receipt() {
    let empty = read_window(json!([])).await.unwrap();
    assert!(empty.is_empty());
    let records: Vec<_> = (1..=100).map(|id| json!({"id":id,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":94})).collect();
    assert!(read_window(json!(records)).await.unwrap().is_empty());
    let mut positive = records.clone();
    positive[99] =
        json!({"id":100,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":95});
    let proof = read_window(json!(positive)).await.unwrap();
    assert_eq!(
        proof.iter().map(|receipt| receipt.id()).collect::<Vec<_>>(),
        vec![100]
    );
    let mut oversized = records;
    oversized.push(
        json!({"id":101,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":95}),
    );
    assert!(read_window(json!(oversized)).await.is_none());
    assert!(
        read_window(json!([
            {"id":1,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":95},
            {"id":2,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":95}
        ]))
        .await
        .is_none()
    );
}
