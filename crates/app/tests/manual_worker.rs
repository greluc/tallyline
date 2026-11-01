// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! App-owned manual storage worker restart contract.

use std::sync::mpsc;
use std::time::Duration;
use tallyline_adapter_storage::api::SqliteReferenceCache;
use tallyline_game::api::model::{EnvironmentMapping, GameEnvironment};
use tallyline_reporting::api::ReportRepository;
use tallyline_reporting::api::model::ReportKey;
use tallyline_shared_kernel::api::model::{TerminalId, Timestamp, TradeSide};
use tallyline_ui::api::model::{BackgroundEvent, ManualCommand};
struct Fixed;
impl tallyline_shared_kernel::api::Clock for Fixed {
    fn now(&self) -> Timestamp {
        Timestamp::from_seconds(1000)
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_ui_16__manual_worker_publishes_only_durable_drafts_and_reopens_them_after_restart() {
    let path =
        std::env::temp_dir().join(format!("tallyline-manual-worker-{}.db", std::process::id()));
    let store = SqliteReferenceCache::open(&path).unwrap();
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(4);
    let worker = tallyline_app::api::ManualWorker::start(
        store,
        Fixed,
        incoming,
        outgoing,
        || Ok(EnvironmentMapping::default()),
        || (),
    )
    .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::ReportsLoaded { selected: None, .. }
    ));
    struct Cache;
    impl tallyline_reference_data::api::ReferenceDataCache for Cache {
        type Error = std::convert::Infallible;
        fn load(
            &mut self,
        ) -> Result<Vec<tallyline_reference_data::api::model::CachedReference>, Self::Error>
        {
            use tallyline_reference_data::api::model::{
                CachedReference, EndpointData, Terminal, TerminalText,
            };
            Ok(vec![CachedReference::new(
                Timestamp::from_seconds(900),
                EndpointData::Terminals(vec![Terminal::new(
                    TerminalId(11),
                    TerminalText {
                        name: "Depot".to_owned(),
                        nickname: "DP".to_owned(),
                        display_name: "Shop".to_owned(),
                        location: "Pyro".to_owned(),
                    },
                    true,
                )]),
            )])
        }
        fn replace(
            &mut self,
            _: &tallyline_reference_data::api::model::CachedReference,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    let snapshot = tallyline_reference_data::api::ReferenceApplication::load(Cache, Fixed)
        .unwrap()
        .snapshot(tallyline_reference_data::api::model::UexEnvironment::Live);
    commands
        .send(ManualCommand::Create {
            key: ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
            snapshot,
        })
        .unwrap();
    let BackgroundEvent::ReportsLoaded { reports, .. } =
        events.recv_timeout(Duration::from_secs(10)).unwrap()
    else {
        panic!("expected durable Draft result");
    };
    let report = reports.first().unwrap().clone();
    worker.finish().unwrap();
    let mut reopened = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(reopened.load_draft(report.id()).unwrap(), Some(report));
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_ui_16__unavailable_mapping_returns_a_passive_failure_instead_of_leaving_the_editor_waiting()
 {
    struct Empty;
    impl tallyline_reference_data::api::ReferenceDataCache for Empty {
        type Error = std::convert::Infallible;
        fn load(
            &mut self,
        ) -> Result<Vec<tallyline_reference_data::api::model::CachedReference>, Self::Error>
        {
            Ok(vec![])
        }
        fn replace(
            &mut self,
            _: &tallyline_reference_data::api::model::CachedReference,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    let snapshot = tallyline_reference_data::api::ReferenceApplication::load(Empty, Fixed)
        .unwrap()
        .snapshot(tallyline_reference_data::api::model::UexEnvironment::Live);
    let path = std::env::temp_dir().join(format!(
        "tallyline-manual-mapping-{}.db",
        std::process::id()
    ));
    let store = SqliteReferenceCache::open(&path).unwrap();
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(4);
    let worker = tallyline_app::api::ManualWorker::start(
        store,
        Fixed,
        incoming,
        outgoing,
        || Err(std::io::Error::other("mapping unavailable")),
        || (),
    )
    .unwrap();
    let _loaded = events.recv_timeout(Duration::from_secs(10)).unwrap();
    commands
        .send(ManualCommand::Create {
            key: ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
            snapshot,
        })
        .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(10)).unwrap(),
        BackgroundEvent::ReportFailed(tallyline_reporting::api::RepositoryError::Unavailable)
    ));
    worker.finish().unwrap();
    std::fs::remove_file(path).unwrap();
}
