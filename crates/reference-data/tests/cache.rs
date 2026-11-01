// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reference application acceptance tests.

use tallyline_reference_data::api::model::{
    CachedReference, Commodity, CommodityAverages, Endpoint, EndpointData, ReferenceState,
    UexEnvironment,
};
use tallyline_reference_data::api::{ReferenceApplication, ReferenceDataCache};
use tallyline_shared_kernel::api::Clock;
use tallyline_shared_kernel::api::model::{CommodityId, Timestamp, TradeSide};

struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_seconds(7_200)
    }
}

struct MemoryCache(Vec<CachedReference>);
impl ReferenceDataCache for MemoryCache {
    type Error = &'static str;
    fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
        Ok(self.0.clone())
    }
    fn replace(&mut self, entry: &CachedReference) -> Result<(), Self::Error> {
        self.0.retain(|old| old.endpoint() != entry.endpoint());
        self.0.push(entry.clone());
        Ok(())
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_ui_13__expired_cached_vocabulary_is_searchable_before_refresh_and_survives_failure() {
    let cached = CachedReference::new(
        Timestamp::from_seconds(100),
        EndpointData::Commodities(vec![Commodity::new(
            CommodityId(42),
            "Agricium",
            "AGRI",
            true,
            CommodityAverages::new(Some("120".parse().unwrap()), None),
        )]),
    );
    let mut application =
        ReferenceApplication::load(MemoryCache(vec![cached]), FixedClock).unwrap();
    assert_eq!(
        application.commodities(UexEnvironment::Live, "agri").len(),
        1
    );
    assert!(application.due().contains(&Endpoint::Commodities));
    application.refresh_failed(Endpoint::Commodities);
    assert_eq!(application.state(), ReferenceState::Offline);
    assert_eq!(
        application
            .commodities(UexEnvironment::Live, "agri")
            .first()
            .unwrap()
            .id(),
        CommodityId(42)
    );
    assert_eq!(
        application.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(100))
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_13__empty_offline_cache_has_an_explicit_no_reference_data_state() {
    let mut application = ReferenceApplication::load(MemoryCache(vec![]), FixedClock).unwrap();
    assert_eq!(application.state(), ReferenceState::NoReferenceData);
    application.refresh_failed(Endpoint::Commodities);
    assert_eq!(application.state(), ReferenceState::NoReferenceData);
    assert!(application.commodities(UexEnvironment::Live, "").is_empty());
}

struct ReadOnlyCache;
impl ReferenceDataCache for ReadOnlyCache {
    type Error = &'static str;
    fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
        Ok(vec![])
    }
    fn replace(&mut self, _: &CachedReference) -> Result<(), Self::Error> {
        Err("read-only")
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__successful_refresh_becomes_visible_only_after_atomic_persistence() {
    let mut application = ReferenceApplication::load(ReadOnlyCache, FixedClock).unwrap();
    let data = EndpointData::Commodities(vec![Commodity::new(
        CommodityId(42),
        "Agricium",
        "AGRI",
        true,
        CommodityAverages::new(Some("120".parse().unwrap()), None),
    )]);
    assert_eq!(application.refreshed(data), Err("read-only"));
    assert!(application.commodities(UexEnvironment::Live, "").is_empty());
    assert_eq!(application.fetched_at(Endpoint::Commodities), None);
    assert_eq!(application.state(), ReferenceState::NoReferenceData);
    let mut application = ReferenceApplication::load(MemoryCache(vec![]), FixedClock).unwrap();
    application
        .refreshed(EndpointData::Commodities(vec![Commodity::new(
            CommodityId(42),
            "Agricium",
            "AGRI",
            true,
            CommodityAverages::new(Some("120".parse().unwrap()), None),
        )]))
        .unwrap();
    assert_eq!(
        application.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(7_200))
    );
    assert_eq!(application.commodities(UexEnvironment::Live, "").len(), 1);
    assert!(!application.due().contains(&Endpoint::Commodities));
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_6__ptu_includes_unavailable_live_vocabulary_without_live_average_substitution() {
    let cached = CachedReference::new(
        Timestamp::from_seconds(100),
        EndpointData::Commodities(vec![
            Commodity::new(
                CommodityId(7),
                "Test Ore",
                "SHARED",
                false,
                CommodityAverages::new(Some("82".parse().unwrap()), None),
            ),
            Commodity::new(
                CommodityId(8),
                "Live Ore",
                "SHARED",
                true,
                CommodityAverages::new(Some("120".parse().unwrap()), None),
            ),
        ]),
    );
    let application = ReferenceApplication::load(MemoryCache(vec![cached]), FixedClock).unwrap();
    assert_eq!(
        application
            .commodities(UexEnvironment::Live, "")
            .iter()
            .map(|item| item.id())
            .collect::<Vec<_>>(),
        vec![CommodityId(8)]
    );
    let items = application.commodities(UexEnvironment::Ptu, "shared");
    assert_eq!(
        items.iter().map(|item| item.id()).collect::<Vec<_>>(),
        vec![CommodityId(7), CommodityId(8)]
    );
    assert!(items.iter().all(|item| {
        item.average_price(UexEnvironment::Ptu, TradeSide::Buy)
            .is_none()
    }));
    assert_eq!(
        items
            .first()
            .unwrap()
            .average_price(UexEnvironment::Live, TradeSide::Buy),
        Some("82".parse().unwrap())
    );
    assert_eq!(
        application.commodities(UexEnvironment::Ptu, "TEST").len(),
        1
    );
    assert!(
        application
            .commodities(UexEnvironment::Live, "missing")
            .is_empty()
    );
}

struct At(u64);
impl Clock for At {
    fn now(&self) -> Timestamp {
        Timestamp::from_seconds(self.0)
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__commodity_refresh_is_due_at_the_one_hour_boundary() {
    for (now, due) in [(3_699, false), (3_700, true)] {
        let entry = CachedReference::new(
            Timestamp::from_seconds(100),
            EndpointData::Commodities(vec![]),
        );
        let application = ReferenceApplication::load(MemoryCache(vec![entry]), At(now)).unwrap();
        assert_eq!(application.due().contains(&Endpoint::Commodities), due);
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__later_success_replaces_old_records_and_clears_offline_state() {
    let old = CachedReference::new(
        Timestamp::from_seconds(100),
        EndpointData::Commodities(vec![Commodity::new(
            CommodityId(1),
            "Old",
            "A",
            true,
            CommodityAverages::default(),
        )]),
    );
    let mut application = ReferenceApplication::load(MemoryCache(vec![old]), FixedClock).unwrap();
    application.refresh_failed(Endpoint::Commodities);
    application
        .refreshed(EndpointData::Commodities(vec![Commodity::new(
            CommodityId(2),
            "New",
            "B",
            true,
            CommodityAverages::default(),
        )]))
        .unwrap();
    assert_eq!(application.state(), ReferenceState::Cached);
    assert_eq!(
        application
            .commodities(UexEnvironment::Live, "")
            .iter()
            .map(|item| item.id())
            .collect::<Vec<_>>(),
        vec![CommodityId(2)]
    );
    let reopened = ReferenceApplication::load(application.into_store(), FixedClock).unwrap();
    assert_eq!(
        reopened
            .commodities(UexEnvironment::Live, "")
            .iter()
            .map(|item| item.id())
            .collect::<Vec<_>>(),
        vec![CommodityId(2)]
    );
    assert_eq!(
        reopened.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(7_200))
    );
}

struct MillisecondClock(u64);
impl Clock for MillisecondClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_millis(self.0)
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__freshness_keeps_milliseconds_and_rechecks_future_dated_cache() {
    for (now, due) in [(3_699_998, false), (3_699_999, true), (99_998, true)] {
        let entry = CachedReference::new(
            Timestamp::from_millis(99_999),
            EndpointData::Commodities(vec![]),
        );
        let application =
            ReferenceApplication::load(MemoryCache(vec![entry]), MillisecondClock(now)).unwrap();
        assert_eq!(application.due().contains(&Endpoint::Commodities), due);
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_api_6_r_ui_8__cached_terminal_search_preserves_identity_and_location_context() {
    use tallyline_reference_data::api::model::{Terminal, TerminalText};
    use tallyline_shared_kernel::api::model::TerminalId;
    let terminals = vec![
        Terminal::new(
            TerminalId(17),
            TerminalText {
                name: "Trade and Development Division".to_owned(),
                nickname: "TDD".to_owned(),
                display_name: "Trade Centre".to_owned(),
                location: "Stanton / ArcCorp / Area18".to_owned(),
            },
            true,
        ),
        Terminal::new(
            TerminalId(18),
            TerminalText {
                name: "Trade and Development Division".to_owned(),
                nickname: "TDD".to_owned(),
                display_name: "Trade Centre".to_owned(),
                location: "Pyro / Test Station".to_owned(),
            },
            false,
        ),
    ];
    let cached = CachedReference::new(
        Timestamp::from_seconds(100),
        EndpointData::Terminals(terminals),
    );
    let mut application =
        ReferenceApplication::load(MemoryCache(vec![cached]), FixedClock).unwrap();
    for query in ["DEVELOPMENT", "tdd", "trade centre", "AREA18"] {
        assert_eq!(
            application
                .terminals(UexEnvironment::Live, query)
                .iter()
                .map(|terminal| terminal.id())
                .collect::<Vec<_>>(),
            vec![TerminalId(17)]
        );
    }
    assert_eq!(
        application
            .terminals(UexEnvironment::Ptu, "tdd")
            .iter()
            .map(|terminal| terminal.id())
            .collect::<Vec<_>>(),
        vec![TerminalId(17), TerminalId(18)]
    );
    assert_eq!(
        application
            .terminals(UexEnvironment::Ptu, "test station")
            .first()
            .unwrap()
            .text()
            .location,
        "Pyro / Test Station"
    );
    assert!(
        application
            .terminals(UexEnvironment::Live, "unmatched")
            .is_empty()
    );
    assert!(!application.due().contains(&Endpoint::Terminals));
    application.refresh_failed(Endpoint::Terminals);
    assert_eq!(application.state(), ReferenceState::Offline);
    assert_eq!(application.terminals(UexEnvironment::Live, "tdd").len(), 1);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__another_endpoint_success_does_not_clear_an_unrecovered_failure() {
    let mut application = ReferenceApplication::load(
        MemoryCache(vec![CachedReference::new(
            Timestamp::from_seconds(100),
            EndpointData::Commodities(vec![]),
        )]),
        FixedClock,
    )
    .unwrap();
    application.refresh_failed(Endpoint::Commodities);
    application
        .refreshed(EndpointData::Terminals(vec![]))
        .unwrap();
    assert_eq!(application.state(), ReferenceState::Offline);
    assert_eq!(
        application.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(100))
    );
    assert_eq!(
        application.fetched_at(Endpoint::Terminals),
        Some(Timestamp::from_seconds(7_200))
    );
    application
        .refreshed(EndpointData::Commodities(vec![]))
        .unwrap();
    assert_eq!(application.state(), ReferenceState::Cached);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__terminal_refresh_expires_at_twelve_hours_independently_of_commodities() {
    for (now, terminal_due) in [(43_299, false), (43_300, true)] {
        let application = ReferenceApplication::load(
            MemoryCache(vec![
                CachedReference::new(
                    Timestamp::from_seconds(100),
                    EndpointData::Terminals(vec![]),
                ),
                CachedReference::new(
                    Timestamp::from_seconds(100),
                    EndpointData::Commodities(vec![]),
                ),
            ]),
            At(now),
        )
        .unwrap();
        assert_eq!(
            application.due().contains(&Endpoint::Terminals),
            terminal_due
        );
        assert!(application.due().contains(&Endpoint::Commodities));
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_cap_3b__cached_versions_remain_read_data_and_successful_fetches_return_fresh_observations() {
    use tallyline_reference_data::api::model::GameVersions;
    let cached = CachedReference::new(
        Timestamp::from_seconds(100),
        EndpointData::GameVersions(GameVersions {
            live: "4.9".to_owned(),
            ptu: Some("3.24.2a".to_owned()),
        }),
    );
    let mut application =
        ReferenceApplication::load(MemoryCache(vec![cached]), FixedClock).unwrap();
    assert_eq!(application.game_versions().unwrap().live, "4.9");
    assert_eq!(
        application.fetched_at(Endpoint::GameVersions),
        Some(Timestamp::from_seconds(100))
    );
    let fetched = GameVersions {
        live: "4.10.2".to_owned(),
        ptu: None,
    };
    let observation = application
        .refreshed(EndpointData::GameVersions(fetched.clone()))
        .unwrap();
    assert_eq!(observation.endpoint, Endpoint::GameVersions);
    assert_eq!(observation.fetched_at, Timestamp::from_seconds(7_200));
    assert_eq!(observation.game_versions, Some(fetched.clone()));
    assert_eq!(application.game_versions(), Some(&fetched));
    let next = application
        .refreshed(EndpointData::GameVersions(fetched.clone()))
        .unwrap();
    assert_eq!(next.game_versions, Some(fetched));
    let vocabulary = application
        .refreshed(EndpointData::Commodities(vec![]))
        .unwrap();
    assert_eq!(vocabulary.game_versions, None);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_sub_5__version_send_freshness_is_fifteen_minutes_and_normal_ttl_is_one_day() {
    use tallyline_reference_data::api::model::GameVersions;
    for (now, send_due, normal_due) in [
        (999, false, false),
        (1_000, true, false),
        (86_499, true, false),
        (86_500, true, true),
    ] {
        let cached = CachedReference::new(
            Timestamp::from_seconds(100),
            EndpointData::GameVersions(GameVersions {
                live: "4.10.2".to_owned(),
                ptu: None,
            }),
        );
        let application = ReferenceApplication::load(MemoryCache(vec![cached]), At(now)).unwrap();
        assert_eq!(
            application
                .due_before_send()
                .contains(&Endpoint::GameVersions),
            send_due
        );
        assert_eq!(
            application.due().contains(&Endpoint::GameVersions),
            normal_due
        );
    }
    let empty = ReferenceApplication::load(MemoryCache(vec![]), At(0)).unwrap();
    assert_eq!(empty.game_versions(), None);
    assert!(empty.due_before_send().contains(&Endpoint::GameVersions));
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__cached_status_levels_keep_side_specific_labels_and_daily_freshness() {
    use std::num::NonZeroU8;
    use tallyline_reference_data::api::model::{StatusLevel, StatusLevels};
    let levels = StatusLevels {
        buy: vec![StatusLevel {
            code: NonZeroU8::new(7).unwrap(),
            name: "Maximum Inventory (Full)".to_owned(),
            short_name: "Maximum".to_owned(),
            abbreviation: "MA".to_owned(),
        }],
        sell: vec![StatusLevel {
            code: NonZeroU8::new(7).unwrap(),
            name: "Maximum Inventory (No Demand)".to_owned(),
            short_name: "Maximum".to_owned(),
            abbreviation: "ND".to_owned(),
        }],
    };
    for (now, due) in [(86_499, false), (86_500, true)] {
        let cached = CachedReference::new(
            Timestamp::from_seconds(100),
            EndpointData::StatusLevels(levels.clone()),
        );
        let application = ReferenceApplication::load(MemoryCache(vec![cached]), At(now)).unwrap();
        assert_eq!(application.status_levels(), Some(&levels));
        assert_eq!(application.due().contains(&Endpoint::StatusLevels), due);
    }
    let mut empty = ReferenceApplication::load(MemoryCache(vec![]), FixedClock).unwrap();
    assert_eq!(empty.status_levels(), None);
    let event = empty
        .refreshed(EndpointData::StatusLevels(levels.clone()))
        .unwrap();
    assert_eq!(event.endpoint, Endpoint::StatusLevels);
    assert_eq!(event.game_versions, None);
    assert_eq!(empty.status_levels(), Some(&levels));
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_6__environment_snapshots_physically_remove_live_averages_and_remain_immutable() {
    let mut application = ReferenceApplication::load(
        MemoryCache(vec![CachedReference::new(
            Timestamp::from_seconds(100),
            EndpointData::Commodities(vec![
                Commodity::new(
                    CommodityId(7),
                    "Test Ore",
                    "SAME",
                    false,
                    CommodityAverages::new(Some("82".parse().unwrap()), None),
                ),
                Commodity::new(
                    CommodityId(8),
                    "Live Ore",
                    "SAME",
                    true,
                    CommodityAverages::new(Some("120".parse().unwrap()), None),
                ),
            ]),
        )]),
        FixedClock,
    )
    .unwrap();
    let ptu = application.snapshot(UexEnvironment::Ptu);
    assert_eq!(ptu.environment(), UexEnvironment::Ptu);
    assert_eq!(ptu.commodities("same").len(), 2);
    assert!(ptu.commodities("").iter().all(|commodity| {
        commodity
            .average_price(UexEnvironment::Live, TradeSide::Buy)
            .is_none()
    }));
    assert_eq!(ptu.commodities("TEST")[0].id(), CommodityId(7));
    assert!(!ptu.commodities("TEST")[0].available(UexEnvironment::Live));
    assert!(ptu.commodities("unmatched").is_empty());
    let live = application.snapshot(UexEnvironment::Live);
    assert_eq!(live.commodities("").len(), 1);
    assert_eq!(
        live.commodities("")[0].average_price(UexEnvironment::Live, TradeSide::Buy),
        Some("120".parse().unwrap())
    );
    application
        .refreshed(EndpointData::Commodities(vec![Commodity::new(
            CommodityId(9),
            "Later Ore",
            "LATER",
            true,
            CommodityAverages::default(),
        )]))
        .unwrap();
    application.refresh_failed(Endpoint::Commodities);
    assert_eq!(ptu.commodities("same").len(), 2);
    assert_eq!(
        ptu.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(100))
    );
    assert_eq!(ptu.state(), ReferenceState::Cached);
    assert_eq!(
        application.snapshot(UexEnvironment::Ptu).state(),
        ReferenceState::Offline
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_api_6__snapshots_search_terminal_context_and_retain_typed_metadata() {
    use std::num::NonZeroU8;
    use tallyline_reference_data::api::model::{
        GameVersions, StatusLevel, StatusLevels, Terminal, TerminalText,
    };
    use tallyline_shared_kernel::api::model::TerminalId;
    let terminals = vec![
        Terminal::new(
            TerminalId(17),
            TerminalText {
                name: "Same".to_owned(),
                nickname: "NICK".to_owned(),
                display_name: "Market".to_owned(),
                location: "Stanton / Area18".to_owned(),
            },
            true,
        ),
        Terminal::new(
            TerminalId(18),
            TerminalText {
                name: "Same".to_owned(),
                nickname: "NICK".to_owned(),
                display_name: "Market".to_owned(),
                location: "Pyro / Test Station".to_owned(),
            },
            false,
        ),
    ];
    let versions = GameVersions {
        live: "4.9".to_owned(),
        ptu: None,
    };
    let levels = StatusLevels {
        buy: vec![StatusLevel {
            code: NonZeroU8::new(7).unwrap(),
            name: "Full".to_owned(),
            short_name: "Full".to_owned(),
            abbreviation: "MA".to_owned(),
        }],
        sell: vec![],
    };
    let application = ReferenceApplication::load(
        MemoryCache(vec![
            CachedReference::new(
                Timestamp::from_seconds(100),
                EndpointData::Terminals(terminals),
            ),
            CachedReference::new(
                Timestamp::from_seconds(110),
                EndpointData::GameVersions(versions.clone()),
            ),
            CachedReference::new(
                Timestamp::from_seconds(120),
                EndpointData::StatusLevels(levels.clone()),
            ),
        ]),
        FixedClock,
    )
    .unwrap();
    let live = application.snapshot(UexEnvironment::Live);
    for query in ["SAME", "nick", "market", "AREA18"] {
        assert_eq!(
            live.terminals(query)
                .iter()
                .map(|terminal| terminal.id())
                .collect::<Vec<_>>(),
            vec![TerminalId(17)]
        );
    }
    assert!(live.terminals("unmatched").is_empty());
    let ptu = application.snapshot(UexEnvironment::Ptu);
    assert_eq!(ptu.terminals("market").len(), 2);
    assert_eq!(ptu.terminals("test station")[0].id(), TerminalId(18));
    assert_eq!(ptu.game_versions(), Some(&versions));
    assert_eq!(ptu.status_levels(), Some(&levels));
    let empty = ReferenceApplication::load(MemoryCache(vec![]), FixedClock)
        .unwrap()
        .snapshot(UexEnvironment::Live);
    assert!(empty.commodities("").is_empty());
    assert!(empty.terminals("").is_empty());
    assert_eq!(empty.game_versions(), None);
    assert_eq!(empty.status_levels(), None);
    assert_eq!(empty.fetched_at(Endpoint::Commodities), None);
    assert_eq!(empty.state(), ReferenceState::NoReferenceData);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_6_r_val_2a__side_specific_exact_averages_are_never_retained_in_ptu_snapshots() {
    use tallyline_reference_data::api::model::CommodityAverages;
    use tallyline_shared_kernel::api::model::TradeSide;
    let buy = "9007199254740993.125".parse().unwrap();
    let sell = "97.25".parse().unwrap();
    let commodity = Commodity::new(
        CommodityId(1),
        "Exact Ore",
        "EX",
        true,
        CommodityAverages::new(Some(buy), Some(sell)),
    );
    let application = ReferenceApplication::load(
        MemoryCache(vec![CachedReference::new(
            Timestamp::from_seconds(1),
            EndpointData::Commodities(vec![commodity]),
        )]),
        FixedClock,
    )
    .unwrap();
    let live = application.snapshot(UexEnvironment::Live);
    let live = live.commodities("");
    assert_eq!(
        live[0].average_price(UexEnvironment::Live, TradeSide::Buy),
        Some(buy)
    );
    assert_eq!(
        live[0].average_price(UexEnvironment::Live, TradeSide::Sell),
        Some(sell)
    );
    let ptu = application.snapshot(UexEnvironment::Ptu);
    for side in [TradeSide::Buy, TradeSide::Sell] {
        assert_eq!(
            ptu.commodities("")[0].average_price(UexEnvironment::Live, side),
            None
        );
        assert_eq!(live[0].average_price(UexEnvironment::Ptu, side), None);
    }
    let zero = "0".parse().unwrap();
    let zero = Commodity::new(
        CommodityId(2),
        "Missing",
        "ZERO",
        true,
        CommodityAverages::new(Some(zero), Some(zero)),
    );
    assert_eq!(
        zero.average_price(UexEnvironment::Live, TradeSide::Buy),
        None
    );
    assert_eq!(
        zero.average_price(UexEnvironment::Live, TradeSide::Sell),
        None
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_6__terminal_container_limit_is_optional_immutable_and_retained_in_snapshots() {
    use tallyline_reference_data::api::model::{Terminal, TerminalText};
    use tallyline_shared_kernel::api::model::TerminalId;
    let original = Terminal::new(
        TerminalId(1),
        TerminalText {
            name: "Trade".to_owned(),
            nickname: String::new(),
            display_name: String::new(),
            location: String::new(),
        },
        true,
    );
    let changed = original
        .clone()
        .with_max_container_size(std::num::NonZeroU32::new(24));
    assert_eq!(original.max_container_size(), None);
    assert_eq!(changed.max_container_size().unwrap().get(), 24);
    let application = ReferenceApplication::load(
        MemoryCache(vec![CachedReference::new(
            Timestamp::from_seconds(1),
            EndpointData::Terminals(vec![changed]),
        )]),
        FixedClock,
    )
    .unwrap();
    for environment in [UexEnvironment::Live, UexEnvironment::Ptu] {
        assert_eq!(
            application.snapshot(environment).terminals("")[0]
                .max_container_size()
                .unwrap()
                .get(),
            24
        );
    }
}
