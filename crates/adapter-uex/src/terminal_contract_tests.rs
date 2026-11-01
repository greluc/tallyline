// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Public terminal-reader contracts against documented wire responses.

use crate::api::{ClientIdentity, ReaderSettings, UexReader};
use serde_json::json;
use tallyline_shared_kernel::api::model::{TerminalId, UexEnvironment};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn r_api_1_r_api_5_r_api_6__terminal_vocabulary_is_public_decoded_once_and_filtered_at_the_boundary()
 {
    let server = MockServer::start().await;
    let mut live = json!({
        "id":"17", "name":"Grey&apos;s Market", "nickname":"Grey&amp;apos;s",
        "displayname":"Trading &amp; Shipping", "type":"commodity",
        "is_player_owned":"0", "is_available_live":"1",
        "star_system_name":"Stanton", "planet_name":"ArcCorp", "orbit_name":null,
        "moon_name":null, "space_station_name":null, "outpost_name":null, "city_name":"Area18",
        "mcs": "deliberately ignored"
    });
    let mut test = live.clone();
    test["id"] = json!(18);
    test["is_available_live"] = json!(0);
    test["planet_name"] = json!("Stanton");
    let mut player_owned = live.clone();
    player_owned["id"] = json!(19);
    player_owned["is_player_owned"] = json!(1);
    let mut refinery = live.clone();
    refinery["id"] = json!(20);
    refinery["type"] = json!("commodity_raw");
    live["max_container_size"] = json!(32);
    Mock::given(method("GET"))
        .and(path("/2.0/terminals/"))
        .and(query_param("type", "commodity"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status":"ok","data":[live,test,player_owned,refinery]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let settings = ReaderSettings::new(
        &format!("{}/2.0/", server.uri()),
        ClientIdentity::new("tallyline-test", "1.2.3"),
    )
    .unwrap();
    let terminals = UexReader::new(&settings)
        .unwrap()
        .terminals()
        .await
        .unwrap();
    assert_eq!(
        terminals
            .iter()
            .map(|terminal| terminal.id())
            .collect::<Vec<_>>(),
        vec![TerminalId(17), TerminalId(18)]
    );
    let first = terminals.first().unwrap();
    assert_eq!(first.text().name, "Grey's Market");
    assert_eq!(first.text().nickname, "Grey&apos;s");
    assert_eq!(first.text().display_name, "Trading & Shipping");
    assert_eq!(first.text().location, "Stanton / ArcCorp / Area18");
    let last = terminals.last().unwrap();
    assert_eq!(last.text().location, "Stanton / Area18");
    assert!(!last.available(UexEnvironment::Live));
    assert!(last.available(UexEnvironment::Ptu));
    let requests = server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|request| !request.headers.contains_key("authorization")
                && !request.headers.contains_key("secret-key"))
    );
}

#[tokio::test]
async fn r_api_5__malformed_or_duplicate_terminal_identity_rejects_the_whole_response() {
    use crate::api::ReadError;
    for change in [
        json!({"id":0}),
        json!({"id":4_294_967_296_u64}),
        json!({"id":17}),
        json!({"id":18,"is_available_live":2}),
        json!({"id":18,"is_player_owned":"invalid"}),
    ] {
        let server = MockServer::start().await;
        let valid = json!({"id":17,"name":"Same","nickname":"Same","displayname":"Same","type":"commodity","is_player_owned":0,"is_available_live":1});
        let mut invalid = valid.clone();
        for (key, value) in change.as_object().unwrap() {
            invalid[key] = value.clone();
        }
        Mock::given(method("GET"))
            .and(path("/2.0/terminals/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"status":"ok","data":[valid,invalid]})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let settings = ReaderSettings::new(
            &format!("{}/2.0/", server.uri()),
            ClientIdentity::new("tallyline-test", "1.2.3"),
        )
        .unwrap();
        assert!(matches!(
            UexReader::new(&settings).unwrap().terminals().await,
            Err(ReadError::Malformed { .. })
        ));
    }
}
