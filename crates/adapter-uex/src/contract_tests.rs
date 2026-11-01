// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Contract tests of the UEX client against responses written from documented and confirmed shapes.

use crate::api::model::{
    CommodityId, ContainerSize, DryRunReport, GameVersions, ReportAccess, StatusLevel, SubmitPrice,
    SubmitSuccess, TerminalId, TradedSide, UexAccount,
};
use crate::api::{
    AppToken, ClientIdentity, ReadError, ReaderSettings, RejectedData, SecretKey, SettingsError,
    UexReader, UexSubmitter,
};
use serde_json::{Value, json};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Outcome = Result<(), String>;

const SECRET_KEY: &str = "test-secret-key-0000";
const APP_TOKEN: &str = "test-app-token-0000";

fn identity() -> ClientIdentity {
    ClientIdentity::new("tallyline-test", "1.2.3")
}

fn settings_for(server: &MockServer) -> Result<ReaderSettings, String> {
    ReaderSettings::new(&format!("{}/2.0/", server.uri()), identity())
        .map_err(|error| format!("{error:?}"))
}

fn reader_with(settings: &ReaderSettings) -> Result<UexReader, String> {
    UexReader::new(settings).map_err(|error| format!("{error:?}"))
}

fn reader_for(server: &MockServer) -> Result<UexReader, String> {
    reader_with(&settings_for(server)?)
}

fn ok(data: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "status": "ok",
        "http_code": 200,
        "data": data,
        "message": ""
    }))
}

fn rejected(http_status: u16, code: &str, message: &str) -> ResponseTemplate {
    ResponseTemplate::new(http_status).set_body_json(json!({
        "status": code,
        "http_code": http_status,
        "data": null,
        "message": message
    }))
}

fn status_level(code: u8, name: &str) -> Value {
    json!({
        "code": code,
        "name": name,
        "name_short": name,
        "name_abbr": "XX",
        "percentage": "0-14%",
        "percentage_start": 0,
        "percentage_end": 14,
        "colors": "red"
    })
}

#[tokio::test]
async fn r_api_7__every_request_carries_the_user_agent_and_the_client_version() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2.0/game_versions/"))
        .and(header("user-agent", "tallyline-test/1.2.3"))
        .and(header("x-client-version", "1.2.3"))
        .respond_with(ok(json!({"live": "4.10.1", "ptu": "4.10.2"})))
        .expect(1)
        .mount(&server)
        .await;

    let versions = reader_for(&server)?.game_versions().await;

    assert_eq!(
        versions,
        Ok(GameVersions {
            live: "4.10.1".to_owned(),
            ptu: Some("4.10.2".to_owned()),
        })
    );
    Ok(())
}

#[tokio::test]
async fn data_submit__dry_run_sends_zero_and_parses_report_ids_in_order() -> Outcome {
    let server = MockServer::start().await;
    let report = DryRunReport::new(
        89,
        "4.10.2".to_owned(),
        1_790_000_000,
        Some("c2NyZWVuc2hvdA==".to_owned()),
        vec![SubmitPrice {
            id_commodity: 24,
            price_buy: Some(136),
            price_sell: None,
            status_buy: Some(2),
            status_sell: None,
            scu_buy: Some(529),
            scu_sell: None,
            container_sizes: Some("1,2,4,8,16".to_owned()),
        }],
    );
    Mock::given(method("POST"))
        .and(path("/2.0/data_submit/"))
        .and(header("authorization", format!("Bearer {APP_TOKEN}")))
        .and(header("secret-key", SECRET_KEY))
        .and(header("x-client-version", "1.2.3"))
        .and(wiremock::matchers::body_json(json!({
            "id_terminal": 89,
            "type": "commodity",
            "is_production": 0,
            "game_version": "4.10.2",
            "date_added": 1_790_000_000_i64,
            "screenshot": "c2NyZWVuc2hvdA==",
            "prices": [{
                "id_commodity": 24,
                "price_buy": 136,
                "status_buy": 2,
                "scu_buy": 529,
                "container_sizes": "1,2,4,8,16"
            }]
        })))
        .respond_with(ok(
            json!({"ids_reports": ["0"], "date_added": 1_790_000_001_i64}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let settings = settings_for(&server)?;
    let submitter = UexSubmitter::new(&settings).map_err(|error| format!("{error:?}"))?;

    let result = submitter
        .dry_run(
            &AppToken::new(APP_TOKEN),
            &SecretKey::new(SECRET_KEY),
            &report,
        )
        .await;

    assert_eq!(
        result,
        Ok(SubmitSuccess {
            ids_reports: vec!["0".to_owned()],
            date_added: 1_790_000_001,
        })
    );
    Ok(())
}

#[tokio::test]
async fn data_submit__validation_error_preserves_zero_based_row_index() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(wiremock::matchers::body_json(json!({
            "id_terminal": 89,
            "type": "commodity",
            "is_production": 0,
            "game_version": "4.10.2",
            "date_added": 1_790_000_000_i64,
            "prices": [
                {
                    "id_commodity": 1,
                    "price_buy": 8379,
                    "status_buy": 3,
                    "scu_buy": 86,
                    "container_sizes": "1,2,4,8,16"
                },
                {
                    "id_commodity": 19,
                    "price_buy": 1,
                    "price_sell": 1
                }
            ]
        })))
        .respond_with(
            rejected(
                400,
                "has_both_price_buy_and_price_sell",
                "Both buy and sell prices are set.",
            )
            .set_body_json(json!({
                "status": "has_both_price_buy_and_price_sell",
                "http_code": 400,
                "data": {"row": 1},
                "message": "Both buy and sell prices are set."
            })),
        )
        .mount(&server)
        .await;
    let submitter =
        UexSubmitter::new(&settings_for(&server)?).map_err(|error| format!("{error:?}"))?;

    let result = submitter
        .dry_run(
            &AppToken::new(APP_TOKEN),
            &SecretKey::new(SECRET_KEY),
            &DryRunReport::new(
                89,
                "4.10.2".to_owned(),
                1_790_000_000,
                None,
                vec![
                    SubmitPrice {
                        id_commodity: 1,
                        price_buy: Some(8379),
                        price_sell: None,
                        status_buy: Some(3),
                        status_sell: None,
                        scu_buy: Some(86),
                        scu_sell: None,
                        container_sizes: Some("1,2,4,8,16".to_owned()),
                    },
                    SubmitPrice {
                        id_commodity: 19,
                        price_buy: Some(1),
                        price_sell: Some(1),
                        status_buy: None,
                        status_sell: None,
                        scu_buy: None,
                        scu_sell: None,
                        container_sizes: None,
                    },
                ],
            ),
        )
        .await;

    assert!(matches!(
        result,
        Err(ReadError::Rejected {
            data: Some(RejectedData::Row(1)),
            ..
        })
    ));
    Ok(())
}

#[tokio::test]
async fn data_submit__duplicate_error_preserves_report_identity() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "status": "duplicated_report",
            "http_code": 400,
            "data": {"report": {"id_terminal": 89, "id_commodity": 24}},
            "message": "Duplicate."
        })))
        .mount(&server)
        .await;
    let submitter =
        UexSubmitter::new(&settings_for(&server)?).map_err(|error| format!("{error:?}"))?;

    let result = submitter
        .dry_run(
            &AppToken::new(APP_TOKEN),
            &SecretKey::new(SECRET_KEY),
            &DryRunReport::new(89, "4.10.2".to_owned(), 1_790_000_000, None, Vec::new()),
        )
        .await;

    assert!(matches!(
        result,
        Err(ReadError::Rejected {
            data: Some(RejectedData::Report {
                id_terminal: 89,
                id_commodity: 24
            }),
            ..
        })
    ));
    Ok(())
}

#[tokio::test]
async fn data_submit__report_level_invalid_date_keeps_its_error_code() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(rejected(400, "invalid_date", "Invalid date."))
        .mount(&server)
        .await;
    let submitter =
        UexSubmitter::new(&settings_for(&server)?).map_err(|error| format!("{error:?}"))?;

    let result = submitter
        .dry_run(
            &AppToken::new(APP_TOKEN),
            &SecretKey::new(SECRET_KEY),
            &DryRunReport::new(89, "4.10.2".to_owned(), 1_790_000_000, None, Vec::new()),
        )
        .await;

    assert!(
        matches!(result, Err(ReadError::Rejected { code, data: None, .. }) if code == "invalid_date")
    );
    Ok(())
}

#[tokio::test]
async fn data_submit__screenshot_check_keeps_its_error_code() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(rejected(400, "screenshot_required", "Screenshot required."))
        .mount(&server)
        .await;
    let submitter =
        UexSubmitter::new(&settings_for(&server)?).map_err(|error| format!("{error:?}"))?;

    let result = submitter
        .dry_run(
            &AppToken::new(APP_TOKEN),
            &SecretKey::new(SECRET_KEY),
            &DryRunReport::new(89, "4.10.2".to_owned(), 1_790_000_000, None, Vec::new()),
        )
        .await;

    assert!(
        matches!(result, Err(ReadError::Rejected { code, data: None, .. }) if code == "screenshot_required")
    );
    Ok(())
}

#[tokio::test]
async fn an_empty_or_missing_ptu_version_means_no_ptu() -> Outcome {
    for ptu in [json!(""), Value::Null] {
        let server = MockServer::start().await;
        Mock::given(path("/2.0/game_versions/"))
            .respond_with(ok(json!({"live": "4.9", "ptu": ptu})))
            .mount(&server)
            .await;

        let versions = reader_for(&server)?.game_versions().await;

        assert_eq!(
            versions,
            Ok(GameVersions {
                live: "4.9".to_owned(),
                ptu: None,
            })
        );
    }
    Ok(())
}

#[tokio::test]
async fn r_api_5__an_error_envelope_becomes_a_rejection_with_its_code() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/game_versions/"))
        .respond_with(rejected(400, "missing_required_input", "Missing input."))
        .mount(&server)
        .await;

    let versions = reader_for(&server)?.game_versions().await;

    assert_eq!(
        versions,
        Err(ReadError::Rejected {
            http_status: 400,
            code: "missing_required_input".to_owned(),
            message: "Missing input.".to_owned(),
            data: None,
        })
    );
    Ok(())
}

#[tokio::test]
async fn r_api_5__an_html_answer_is_reported_as_not_json() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/game_versions/"))
        .respond_with(
            ResponseTemplate::new(404)
                .insert_header("content-type", "text/html")
                .set_body_string("<!DOCTYPE html><html><body>Not found</body></html>"),
        )
        .mount(&server)
        .await;

    let versions = reader_for(&server)?.game_versions().await;

    assert_eq!(versions, Err(ReadError::NotJson { http_status: 404 }));
    Ok(())
}

#[tokio::test]
async fn r_api_5__an_ok_envelope_with_data_of_the_wrong_shape_is_malformed() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/game_versions/"))
        .respond_with(ok(json!(["4.10.1"])))
        .mount(&server)
        .await;

    let versions = reader_for(&server)?.game_versions().await;

    assert!(matches!(versions, Err(ReadError::Malformed { .. })));
    Ok(())
}

#[tokio::test]
async fn r_api_5__a_body_above_the_size_limit_is_refused() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/game_versions/"))
        .respond_with(ok(json!({"live": "4.10.1", "ptu": "4.10.2"})))
        .mount(&server)
        .await;
    let settings = settings_for(&server)?
        .with_max_body_bytes(16)
        .map_err(|error| format!("{error:?}"))?;

    let versions = reader_with(&settings)?.game_versions().await;

    assert_eq!(versions, Err(ReadError::BodyTooLarge { limit: 16 }));
    Ok(())
}

#[test]
fn a_zero_body_limit_is_refused() -> Outcome {
    let settings = ReaderSettings::new("https://api.uexcorp.space/2.0/", identity())
        .map_err(|error| format!("{error:?}"))?
        .with_max_body_bytes(0);

    assert_eq!(settings.err(), Some(SettingsError::ZeroBodyLimit));
    Ok(())
}

#[tokio::test]
async fn r_api_3__a_redirect_is_not_followed() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/game_versions/"))
        .respond_with(
            ResponseTemplate::new(302).insert_header("location", "https://example.invalid/"),
        )
        .mount(&server)
        .await;

    let versions = reader_for(&server)?.game_versions().await;

    assert_eq!(versions, Err(ReadError::Redirect { http_status: 302 }));
    Ok(())
}

#[test]
fn r_api_3__plain_http_is_refused_for_a_host_other_than_loopback() {
    let settings = ReaderSettings::new("http://api.uexcorp.space/2.0/", identity());

    assert_eq!(settings.err(), Some(SettingsError::InsecureScheme));
}

#[test]
fn r_api_3__https_is_accepted_for_the_documented_hosts() {
    for base in [
        "https://api.uexcorp.space/2.0/",
        "https://api.uexcorp.uk/2.0/",
    ] {
        assert!(ReaderSettings::new(base, identity()).is_ok(), "{base}");
    }
}

#[test]
fn a_base_url_without_a_trailing_slash_is_refused() {
    let settings = ReaderSettings::new("https://api.uexcorp.space/2.0", identity());

    assert_eq!(settings.err(), Some(SettingsError::InvalidBaseUrl));
}

#[test]
fn a_reader_is_built_with_the_explicit_crypto_provider() -> Outcome {
    let settings = ReaderSettings::new("https://api.uexcorp.space/2.0/", identity())
        .map_err(|error| format!("{error:?}"))?;

    assert!(UexReader::new(&settings).is_ok());
    Ok(())
}

#[tokio::test]
async fn status_levels_are_read_per_side_with_their_own_names() -> Outcome {
    let server = MockServer::start().await;
    let buy: Vec<Value> = (1..=7)
        .map(|code| status_level(code, &format!("Buy level {code}")))
        .collect();
    let sell: Vec<Value> = (1..=7)
        .map(|code| status_level(code, &format!("Sell level {code}")))
        .collect();
    Mock::given(path("/2.0/commodities_status/"))
        .respond_with(ok(json!({"buy": buy, "sell": sell})))
        .mount(&server)
        .await;

    let levels = reader_for(&server)?
        .commodity_status_levels()
        .await
        .map_err(|error| format!("{error:?}"))?;

    assert_eq!(levels.buy.len(), 7);
    assert_eq!(levels.sell.len(), 7);
    assert_eq!(
        levels.sell.last(),
        Some(&StatusLevel {
            code: 7,
            name: "Sell level 7".to_owned(),
            name_short: "Sell level 7".to_owned(),
            name_abbr: "XX".to_owned(),
        })
    );
    Ok(())
}

#[tokio::test]
async fn r_api_5__a_status_level_of_zero_is_malformed() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/commodities_status/"))
        .respond_with(ok(json!({"buy": [status_level(0, "None")], "sell": []})))
        .mount(&server)
        .await;

    let levels = reader_for(&server)?.commodity_status_levels().await;

    assert!(matches!(levels, Err(ReadError::Malformed { .. })));
    Ok(())
}

#[tokio::test]
async fn r_api_5__html_escaped_names_are_unescaped_once() -> Outcome {
    let server = MockServer::start().await;
    let level = json!({
        "code": 1,
        "name": "Grey&apos;s &amp;amp; &lt;Market&gt;",
        "name_short": "A &amp B",
        "name_abbr": "&quot;GM&#39;"
    });
    Mock::given(path("/2.0/commodities_status/"))
        .respond_with(ok(json!({"buy": [level], "sell": []})))
        .mount(&server)
        .await;

    let levels = reader_for(&server)?
        .commodity_status_levels()
        .await
        .map_err(|error| format!("{error:?}"))?;

    assert_eq!(
        levels.buy,
        vec![StatusLevel {
            code: 1,
            name: "Grey's &amp; <Market>".to_owned(),
            name_short: "A &amp B".to_owned(),
            name_abbr: "\"GM'".to_owned(),
        }]
    );
    Ok(())
}
#[tokio::test]
async fn price_rows_carry_their_traded_side_and_container_sizes() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/commodities_prices/"))
        .and(query_param("id_terminal", "89"))
        .respond_with(ok(json!([
            {
                "id_terminal": 89, "id_commodity": 1,
                "price_buy": 0, "price_sell": 120,
                "status_buy": 0, "status_sell": 2,
                "container_sizes": "1,2,4,8,16,24,32"
            },
            {
                "id_terminal": 89, "id_commodity": 24,
                "price_buy": 136, "price_sell": 0,
                "status_buy": 1, "status_sell": 0,
                "container_sizes": ""
            },
            {
                "id_terminal": 89, "id_commodity": 30,
                "price_buy": 0, "price_sell": 0,
                "status_buy": 0, "status_sell": 0,
                "container_sizes": "32"
            }
        ])))
        .mount(&server)
        .await;

    let rows = reader_for(&server)?
        .price_rows(TerminalId(89))
        .await
        .map_err(|error| format!("{error:?}"))?;

    let shape: Vec<(CommodityId, TradedSide, Option<u8>, Vec<ContainerSize>)> = rows
        .into_iter()
        .map(|row| (row.commodity, row.side, row.status, row.container_sizes))
        .collect();
    assert_eq!(
        shape,
        vec![
            (
                CommodityId(1),
                TradedSide::Sell,
                Some(2),
                vec![
                    ContainerSize::Scu1,
                    ContainerSize::Scu2,
                    ContainerSize::Scu4,
                    ContainerSize::Scu8,
                    ContainerSize::Scu16,
                    ContainerSize::Scu24,
                    ContainerSize::Scu32,
                ]
            ),
            (CommodityId(24), TradedSide::Buy, Some(1), vec![]),
            (
                CommodityId(30),
                TradedSide::Neither,
                None,
                vec![ContainerSize::Scu32]
            ),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn r_api_5__numbers_and_flags_sent_as_strings_are_accepted() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/commodities_prices/"))
        .respond_with(ok(json!([{
            "id_terminal": "89", "id_commodity": "1",
            "price_buy": "0", "price_sell": "120",
            "status_buy": "0", "status_sell": "2",
            "container_sizes": "16,32"
        }])))
        .mount(&server)
        .await;

    let rows = reader_for(&server)?.price_rows(TerminalId(89)).await;

    assert!(
        matches!(rows.as_deref(), Ok([row]) if row.commodity == CommodityId(1) && row.side == TradedSide::Sell),
        "{rows:?}"
    );
    Ok(())
}

#[tokio::test]
async fn r_api_5__a_container_size_outside_the_documented_set_is_malformed() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/commodities_prices/"))
        .respond_with(ok(json!([{
            "id_terminal": 89, "id_commodity": 1,
            "price_buy": 0, "price_sell": 120,
            "status_buy": 0, "status_sell": 2,
            "container_sizes": "1,3"
        }])))
        .mount(&server)
        .await;

    let rows = reader_for(&server)?.price_rows(TerminalId(89)).await;

    assert!(matches!(rows, Err(ReadError::Malformed { .. })), "{rows:?}");
    Ok(())
}

#[tokio::test]
async fn r_api_4__the_account_check_sends_the_secret_key_and_no_app_token() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2.0/user/"))
        .and(header("secret-key", SECRET_KEY))
        .respond_with(ok(json!({
            "username": "datarunner-one",
            "is_datarunner": 1,
            "is_datarunner_banned": 0,
            "is_staff": 0,
            "date_added": 1700000000,
            "email": "someone@example.invalid",
            "discord_username": "someone#0001"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let account = reader_for(&server)?
        .account(&SecretKey::new(SECRET_KEY))
        .await;

    assert_eq!(
        account,
        Ok(UexAccount {
            username: "datarunner-one".to_owned(),
            is_datarunner: true,
            is_datarunner_banned: false,
            is_staff: false,
        })
    );
    let requests = server.received_requests().await.unwrap_or_default();
    assert_eq!(requests.len(), 1);
    assert!(
        requests
            .iter()
            .all(|request| !request.headers.contains_key("authorization"))
    );
    Ok(())
}

#[tokio::test]
async fn r_api_4__an_invalid_secret_key_is_rejected_with_its_code() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/user/"))
        .respond_with(rejected(403, "invalid_secret_key", ""))
        .mount(&server)
        .await;

    let account = reader_for(&server)?
        .account(&SecretKey::new(SECRET_KEY))
        .await;

    assert_eq!(
        account,
        Err(ReadError::Rejected {
            http_status: 403,
            code: "invalid_secret_key".to_owned(),
            message: String::new(),
            data: None,
        })
    );
    Ok(())
}

#[tokio::test]
async fn r_api_4__report_access_sends_the_bearer_token_and_the_secret_key() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2.0/data_info/"))
        .and(query_param("username", "datarunner-one"))
        .and(query_param("limit", "1"))
        .and(header(
            "authorization",
            format!("Bearer {APP_TOKEN}").as_str(),
        ))
        .and(header("secret-key", SECRET_KEY))
        .respond_with(ok(json!([{"id": 964307, "username": "datarunner-one"}])))
        .expect(1)
        .mount(&server)
        .await;

    let access = reader_for(&server)?
        .report_access(
            &AppToken::new(APP_TOKEN),
            &SecretKey::new(SECRET_KEY),
            "datarunner-one",
        )
        .await;

    assert_eq!(access, Ok(ReportAccess { rows: 1 }));
    Ok(())
}

#[tokio::test]
async fn r_api_4__a_refused_app_token_keeps_the_uex_code() -> Outcome {
    let server = MockServer::start().await;
    Mock::given(path("/2.0/data_info/"))
        .respond_with(rejected(403, "access_denied", "Invalid bearer token."))
        .mount(&server)
        .await;

    let access = reader_for(&server)?
        .report_access(
            &AppToken::new(APP_TOKEN),
            &SecretKey::new(SECRET_KEY),
            "datarunner-one",
        )
        .await;

    assert_eq!(
        access,
        Err(ReadError::Rejected {
            http_status: 403,
            code: "access_denied".to_owned(),
            message: "Invalid bearer token.".to_owned(),
            data: None,
        })
    );
    Ok(())
}

#[test]
fn credentials_are_masked_in_debug_output() {
    let rendered = format!(
        "{:?} {:?}",
        SecretKey::new(SECRET_KEY),
        AppToken::new(APP_TOKEN)
    );

    assert!(!rendered.contains(SECRET_KEY), "{rendered}");
    assert!(!rendered.contains(APP_TOKEN), "{rendered}");
}
