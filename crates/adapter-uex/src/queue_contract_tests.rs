// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Queue gateway contracts using an isolated local HTTP server.

use crate::api::{ClientIdentity, ReaderSettings, UexReader};
use serde_json::json;
use tallyline_shared_kernel::api::model::{
    CommodityId, ReportId, RowId, SubmissionMode, TerminalId, Timestamp, TradeSide, UexEnvironment,
};
use tallyline_submission::api::model::{
    CredentialValues, SubmissionHeader, SubmissionRequest, SubmissionResult, SubmissionRow,
};
use tallyline_submission::api::{SubmissionGateway, SubmissionReceiptVerifier};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_1_r_sub_8_r_api_7__queue_dry_run_sends_exact_one_sided_values_and_omits_zero_scu() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/2.0/data_submit/"))
        .and(header("user-agent", "tallyline-test/1.2.3"))
        .and(header("x-client-version", "1.2.3"))
        .and(header("secret-key", "dummy-key"))
        .and(header("authorization", "Bearer dummy-token"))
        .and(body_json(json!({"id_terminal":89,"type":"commodity","is_production":0,"game_version":"4.10.2","date_added":100,"prices":[{"id_commodity":7,"price_sell":120.125,"status_sell":1}]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok","data":{"ids_reports":null,"date_added":100}})))
        .expect(1).mount(&server).await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let request = SubmissionRequest::new(
        SubmissionHeader {
            report: ReportId(1),
            report_version: 3,
            account: "dummy-a".to_owned(),
            mode: SubmissionMode::DryRun,
            terminal: TerminalId(89),
            side: TradeSide::Sell,
            environment: UexEnvironment::Live,
            game_version: "4.10.2".to_owned(),
            observed_at: Timestamp::from_seconds(100),
        },
        vec![
            SubmissionRow::new(
                RowId(1),
                CommodityId(7),
                "120.125".parse().unwrap(),
                "0".parse().unwrap(),
                1,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    assert_eq!(
        reader
            .submit(reader.prepare(&request).unwrap().0, &credentials, None)
            .await,
        SubmissionResult::DryRunSucceeded
    );
    server.verify().await;
}

fn released_request() -> Option<SubmissionRequest> {
    SubmissionRequest::new(
        SubmissionHeader {
            report: ReportId(1),
            report_version: 3,
            account: "dummy-a".to_owned(),
            mode: SubmissionMode::DryRun,
            terminal: TerminalId(89),
            side: TradeSide::Sell,
            environment: UexEnvironment::Live,
            game_version: "4.10.2".to_owned(),
            observed_at: Timestamp::from_seconds(100),
        },
        vec![
            SubmissionRow::new(
                RowId(1),
                CommodityId(7),
                "120.125".parse().ok()?,
                "0".parse().ok()?,
                1,
            )
            .ok()?,
        ],
    )
    .ok()
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_9__refused_connection_is_not_processed_but_bad_response_stays_unknown() {
    use tallyline_submission::api::model::NotProcessed;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let settings = ReaderSettings::for_test_server(
        &format!("http://{address}/2.0/"),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    assert_eq!(
        reader
            .submit(
                reader.prepare(&released_request().unwrap()).unwrap().0,
                &credentials,
                None
            )
            .await,
        SubmissionResult::NotProcessed(NotProcessed::ConnectFailure)
    );
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("broken reply"))
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
    assert_eq!(
        reader
            .submit(
                reader.prepare(&released_request().unwrap()).unwrap().0,
                &credentials,
                None
            )
            .await,
        SubmissionResult::Unknown
    );
    server.verify().await;
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_man_5_r_sub_9__dry_run_screenshot_requirement_survives_transport_translation() {
    use tallyline_submission::api::model::SubmissionReturnReason;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({"status":"screenshot_required","data":null})),
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
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    assert_eq!(
        reader
            .submit(
                reader.prepare(&released_request().unwrap()).unwrap().0,
                &credentials,
                None
            )
            .await,
        SubmissionResult::Returned(SubmissionReturnReason::ScreenshotRequired)
    );
    server.verify().await;
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_9__production_ids_are_verified_by_identity_and_never_mapped_by_position() {
    use wiremock::matchers::query_param;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"status":"ok","data":{"ids_reports":["11","12"],"date_added":100}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    for (id, commodity) in [(11, 8), (12, 7)] {
        Mock::given(method("GET")).and(path("/2.0/data_info/")).and(query_param("id",id.to_string())).and(query_param("username","dummy-a"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok","data":[{"id":id,"type":"commodity","id_terminal":89,"id_commodity":commodity,"date_added":100}]})))
            .expect(1).mount(&server).await;
    }
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let base = released_request().unwrap();
    let mut header = base.header().clone();
    header.mode = SubmissionMode::Production;
    let mut rows = base.rows().to_vec();
    rows.push(
        SubmissionRow::new(
            RowId(2),
            CommodityId(8),
            "2".parse().unwrap(),
            "10".parse().unwrap(),
            2,
        )
        .unwrap(),
    );
    let request = SubmissionRequest::new(header, rows).unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let result = reader
        .submit(reader.prepare(&request).unwrap().0, &credentials, None)
        .await;
    let accepted = accepted_job(&request, result, Timestamp::from_seconds(100)).unwrap();
    let receipts = reader
        .verify_receipts(&accepted, &credentials, None)
        .await
        .unwrap();
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0].row(), RowId(2));
    assert_eq!(receipts[0].id(), 11);
    assert_eq!(receipts[1].row(), RowId(1));
    assert_eq!(receipts[1].id(), 12);
    assert_eq!(receipts[0].date_added(), Timestamp::from_seconds(100));
    server.verify().await;
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_9__partial_ids_remain_unmapped_with_their_returned_partition_time() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"status":"ok","data":{"ids_reports":["11"],"date_added":100}}),
            ),
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
    let base = released_request().unwrap();
    let mut header = base.header().clone();
    header.mode = SubmissionMode::Production;
    let mut rows = base.rows().to_vec();
    rows.push(
        SubmissionRow::new(
            RowId(2),
            CommodityId(8),
            "2".parse().unwrap(),
            "10".parse().unwrap(),
            2,
        )
        .unwrap(),
    );
    let request = SubmissionRequest::new(header, rows).unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let SubmissionResult::Partial(unverified) = reader
        .submit(reader.prepare(&request).unwrap().0, &credentials, None)
        .await
    else {
        panic!("partial receipt count must fail closed");
    };
    assert_eq!(unverified.len(), 1);
    assert_eq!(unverified[0].id(), 11);
    assert_eq!(unverified[0].date_added(), Timestamp::from_seconds(100));
    assert!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .all(|request| request.method.as_str() == "POST")
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_sub_2__payload_is_serialised_and_hashed_before_the_async_attempt_begins() {
    let settings = ReaderSettings::new(
        "https://api.uexcorp.uk/2.0/",
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap();
    let reader = UexReader::new(&settings).unwrap();
    let (_, hash) = reader.prepare(&released_request().unwrap()).unwrap();
    let actual: String = hash
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(
        actual,
        "05f8a6292d79e9a33b014a01c7d13059aac8d124a238f9d7e59ee0bfbcac9e0c"
    );
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_8_r_sub_9__malformed_or_positive_dry_run_receipts_cannot_be_success() {
    for data in [
        json!({"ids_reports":["0"],"date_added":"broken"}),
        json!({"ids_reports":["42"],"date_added":100}),
        json!({"ids_reports":["+0"],"date_added":100}),
        json!({"ids_reports":["0","0"],"date_added":100}),
        json!({"ids_reports":["0"],"date_added":101}),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"status":"ok","data":data})),
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
        let request = released_request().unwrap();
        let (prepared, _) = reader.prepare(&request).unwrap();
        let values =
            CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
        assert_eq!(
            reader.submit(prepared, &values, None).await,
            SubmissionResult::Unknown
        );
        server.verify().await;
    }
}

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;
struct ServerClock {
    tick: AtomicU64,
}
impl tallyline_shared_kernel::api::Clock for ServerClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_seconds(784111900)
    }
}
impl tallyline_submission::api::SubmissionClock for ServerClock {
    fn monotonic(&self) -> Duration {
        Duration::from_millis(self.tick.load(Ordering::SeqCst))
    }
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_val_6__every_supported_http_date_format_clamps_the_prepared_wire_time_with_monotonic_expiry()
 {
    use crate::api::model::ServerTimeTracker;
    for date in [
        "Sun, 06 Nov 1994 08:49:37 GMT",
        "Sunday, 06-Nov-94 08:49:37 GMT",
        "Sun Nov  6 08:49:37 1994",
    ] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/2.0/game_versions/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("date", date)
                    .set_body_json(json!({"status":"ok","data":{"live":"4.10.2","ptu":null}})),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST")).and(body_json(json!({"id_terminal":89,"type":"commodity","is_production":0,"game_version":"4.10.2","date_added":784111779,"prices":[{"id_commodity":7,"price_sell":120.125,"status_sell":1}]}))).respond_with(ResponseTemplate::new(200).insert_header("date",date).set_body_json(json!({"status":"ok","data":{"ids_reports":["0"],"date_added":784111779}}))).expect(1).mount(&server).await;
        let clock = Arc::new(ServerClock {
            tick: AtomicU64::new(50000),
        });
        let settings = ReaderSettings::for_test_server(
            &format!("{}/2.0/", server.uri()),
            ClientIdentity::new("tallyline-test", "1.2.3"),
        )
        .unwrap()
        .with_expert_host_confirmation();
        let reader =
            UexReader::new_with_server_time(&settings, ServerTimeTracker::new(clock.clone()))
                .unwrap();
        reader.game_versions().await.unwrap();
        clock.tick.store(52250, Ordering::SeqCst);
        let base = released_request().unwrap();
        let mut header = base.header().clone();
        header.observed_at = Timestamp::from_seconds(784111900);
        let request = SubmissionRequest::new(header, base.rows().to_vec()).unwrap();
        let (prepared, _) = reader.prepare(&request).unwrap();
        assert_eq!(prepared.date_added(), Timestamp::from_seconds(784111779));
        let values =
            CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
        assert_eq!(
            reader.submit(prepared, &values, None).await,
            SubmissionResult::DryRunSucceeded
        );
        clock.tick.store(1000001, Ordering::SeqCst);
        assert_eq!(
            reader.prepare(&request).unwrap().0.date_added(),
            Timestamp::from_seconds(784111900)
        );
        assert_eq!(
            request.header().observed_at,
            Timestamp::from_seconds(784111900)
        );
        server.verify().await;
    }
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_val_6_r_sub_9__production_lookup_matches_the_actual_clamped_wire_date() {
    use crate::api::model::ServerTimeTracker;
    use wiremock::matchers::query_param;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2.0/game_versions/"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("date", "Thu, 01 Jan 1970 00:01:35 GMT")
                .set_body_json(json!({"status":"ok","data":{"live":"4.10.2","ptu":null}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST")).and(body_json(json!({"id_terminal":89,"type":"commodity","is_production":1,"game_version":"4.10.2","date_added":95,"prices":[{"id_commodity":7,"price_sell":120.125,"status_sell":1}]}))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok","data":{"ids_reports":["42"],"date_added":95}}))).expect(1).mount(&server).await;
    Mock::given(method("GET")).and(path("/2.0/data_info/")).and(query_param("id","42")).and(query_param("username","dummy-a")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok","data":[{"id":42,"type":"commodity","id_terminal":89,"id_commodity":7,"date_added":95}]}))).expect(1).mount(&server).await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new_with_server_time(
        &settings,
        ServerTimeTracker::new(Arc::new(ServerClock {
            tick: AtomicU64::new(50000),
        })),
    )
    .unwrap();
    reader.game_versions().await.unwrap();
    let base = released_request().unwrap();
    let mut header = base.header().clone();
    header.mode = SubmissionMode::Production;
    let request = SubmissionRequest::new(header, base.rows().to_vec()).unwrap();
    let (prepared, _) = reader.prepare(&request).unwrap();
    let values =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let result = reader.submit(prepared, &values, None).await;
    let accepted = accepted_job(&request, result, Timestamp::from_seconds(95)).unwrap();
    let receipts = reader
        .verify_receipts(&accepted, &values, None)
        .await
        .unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].id(), 42);
    assert_eq!(receipts[0].date_added(), Timestamp::from_seconds(95));
    assert_eq!(request.header().observed_at, Timestamp::from_seconds(100));
    server.verify().await;
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_9__full_count_success_survives_unavailable_identity_lookup_without_positional_mapping()
 {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"status":"ok","data":{"ids_reports":["42"],"date_added":100}}),
            ),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(503))
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
    let base = released_request().unwrap();
    let mut header = base.header().clone();
    header.mode = SubmissionMode::Production;
    let request = SubmissionRequest::new(header, base.rows().to_vec()).unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let result = reader
        .submit(reader.prepare(&request).unwrap().0, &credentials, None)
        .await;
    let accepted = accepted_job(&request, result.clone(), Timestamp::from_seconds(100)).unwrap();
    assert!(
        reader
            .verify_receipts(&accepted, &credentials, None)
            .await
            .is_none()
    );
    assert_eq!(
        accepted.recover(Timestamp::from_seconds(120)).unwrap(),
        accepted
    );
    let SubmissionResult::ProductionAccepted(ids) = result else {
        panic!("full-count acceptance must remain successful while verification is unavailable");
    };
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0].id(), 42);
    assert_eq!(ids[0].date_added(), Timestamp::from_seconds(100));
    server.verify().await;
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_api_5_r_sub_8__legacy_single_numeric_string_and_null_test_receipts_remain_supported() {
    for ids in [json!("0"), serde_json::Value::Null] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"status":"ok","data":{"ids_reports":ids,"date_added":"100"}}),
            ))
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
        let request = released_request().unwrap();
        let credentials =
            CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
        assert_eq!(
            reader
                .submit(reader.prepare(&request).unwrap().0, &credentials, None)
                .await,
            SubmissionResult::DryRunSucceeded
        );
        server.verify().await;
    }
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_7_r_nf_4__prepared_screenshot_is_bounded_plain_base64_and_part_of_the_exact_attempt_digest()
 {
    use crate::api::model::PreparedUexSubmission;
    use tallyline_submission::api::model::PreparationFailure;
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(body_json(json!({"id_terminal":89,"type":"commodity","is_production":0,"game_version":"4.10.2","date_added":100,"prices":[{"id_commodity":7,"price_sell":120.125,"status_sell":1}],"screenshot":"AP8="}))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok","data":{"ids_reports":null,"date_added":100}}))).expect(1).mount(&server).await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let request = released_request().unwrap();
    let (_, no_image) = reader.prepare(&request).unwrap();
    let (prepared, image_hash) = PreparedUexSubmission::new_with_screenshot(
        &request,
        Timestamp::from_seconds(100),
        Some("AP8="),
    )
    .unwrap();
    assert_ne!(image_hash, no_image);
    for invalid in [
        "",
        "data:image/png;base64,AP8=",
        "AP8=\n",
        "A===",
        "AP9=",
        "AP8",
    ] {
        assert!(matches!(
            PreparedUexSubmission::new_with_screenshot(
                &request,
                Timestamp::from_seconds(100),
                Some(invalid)
            ),
            Err(PreparationFailure::InvalidPayload)
        ));
    }
    let limit = "A".repeat(10_000_000);
    assert!(
        PreparedUexSubmission::new_with_screenshot(
            &request,
            Timestamp::from_seconds(100),
            Some(&limit)
        )
        .is_ok()
    );
    let excessive = "A".repeat(10_000_004);
    assert!(
        PreparedUexSubmission::new_with_screenshot(
            &request,
            Timestamp::from_seconds(100),
            Some(&excessive)
        )
        .is_err()
    );
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    assert_eq!(
        reader.submit(prepared, &credentials, None).await,
        SubmissionResult::DryRunSucceeded
    );
    server.verify().await;
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_val_6_r_api_5__public_metadata_observes_server_time_before_payload_or_transport_refusal()
{
    use crate::api::model::{PublicContractEndpoint, ServerTimeTracker};
    for status in [200, 302, 503] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/2.0/game_versions/"))
            .respond_with(
                ResponseTemplate::new(status)
                    .insert_header("date", "Sun, 06 Nov 1994 08:49:37 GMT")
                    .set_body_string("invalid private response"),
            )
            .expect(1)
            .mount(&server)
            .await;
        let tracker = ServerTimeTracker::new(Arc::new(ServerClock {
            tick: AtomicU64::new(0),
        }));
        let settings = ReaderSettings::for_test_server(
            &format!("{}/2.0/", server.uri()),
            ClientIdentity::new("tallyline-test", "1.2.3"),
        )
        .unwrap();
        let reader = UexReader::new_with_server_time(&settings, tracker.clone()).unwrap();
        match reader
            .public_contract_sample(PublicContractEndpoint::GameVersions)
            .await
        {
            Ok(payload) => assert!(payload.into_schema().is_err()),
            Err(_) => assert_eq!(status, 302),
        }
        assert_eq!(
            tracker.clamp(Timestamp::from_seconds(784111900)),
            Timestamp::from_seconds(784111777)
        );
        server.verify().await;
    }
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_val_6_r_sub_11__invalid_date_returns_for_review_without_changing_the_observation() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2.0/data_submit/"))
        .respond_with(ResponseTemplate::new(400).set_body_json(
            json!({"status":"invalid_date","message":"Check observation time","data":null}),
        ))
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
    let request = released_request().unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let result = reader
        .submit(reader.prepare(&request).unwrap().0, &credentials, None)
        .await;
    assert!(matches!(result, SubmissionResult::Returned(_)));
    assert_eq!(request.header().observed_at, Timestamp::from_seconds(100));
    server.verify().await;
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_2_r_sub_9__full_count_acceptance_returns_before_any_identity_lookup() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2.0/data_submit/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"status":"ok","data":{"ids_reports":["42"],"date_added":100}}),
            ),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(503))
        .expect(0)
        .mount(&server)
        .await;
    let settings = ReaderSettings::for_test_server(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap()
    .with_expert_host_confirmation();
    let reader = UexReader::new(&settings).unwrap();
    let base = released_request().unwrap();
    let mut header = base.header().clone();
    header.mode = SubmissionMode::Production;
    let request = SubmissionRequest::new(header, base.rows().to_vec()).unwrap();
    let credentials =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    let result = reader
        .submit(reader.prepare(&request).unwrap().0, &credentials, None)
        .await;
    assert!(matches!(result, SubmissionResult::ProductionAccepted(_)));
    server.verify().await;
}

fn accepted_job(
    request: &SubmissionRequest,
    result: SubmissionResult,
    date_added: Timestamp,
) -> Option<tallyline_submission::api::model::SubmissionJob> {
    use tallyline_submission::api::model::{JobId, PayloadHash, SubmissionJob};
    SubmissionJob::queued(JobId(1), request.clone())
        .ok()?
        .begin_prepared(
            Timestamp::from_seconds(110),
            PayloadHash::new([1; 32]),
            date_added,
        )
        .ok()?
        .complete(result, Timestamp::from_seconds(115))
        .ok()
}

#[tokio::test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
async fn r_sub_1_r_man_2__zero_scu_above_status_one_is_omitted_without_rounding_the_entered_price()
{
    let server = MockServer::start().await;
    let expected: serde_json::Value = serde_json::from_str(r#"{"id_terminal":89,"type":"commodity","is_production":0,"game_version":"4.10.2","date_added":100,"prices":[{"id_commodity":7,"price_sell":12.123456789012345678901234567,"status_sell":3}]}"#).unwrap();
    Mock::given(method("POST"))
        .and(path("/2.0/data_submit/"))
        .and(body_json(expected))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status":"ok","data":{"ids_reports":null,"date_added":100}})),
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
    let base = released_request().unwrap();
    let request = SubmissionRequest::new(
        base.header().clone(),
        vec![
            SubmissionRow::new(
                RowId(1),
                CommodityId(7),
                "12.123456789012345678901234567".parse().unwrap(),
                "0".parse().unwrap(),
                3,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let values =
        CredentialValues::new("dummy-key".to_owned(), Some("dummy-token".to_owned())).unwrap();
    assert_eq!(
        reader
            .submit(reader.prepare(&request).unwrap().0, &values, None)
            .await,
        SubmissionResult::DryRunSucceeded
    );
    let requests = server.received_requests().await.unwrap();
    let body = std::str::from_utf8(&requests[0].body).unwrap();
    assert!(body.contains("12.123456789012345678901234567"));
    assert!(!body.contains("scu_sell"));
    server.verify().await;
}
