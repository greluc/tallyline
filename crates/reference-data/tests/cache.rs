// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reference application acceptance tests.

use tallyline_reference_data::api::model::{
    CachedReference, Commodity, Endpoint, EndpointData, ReferenceState, UexEnvironment,
};
use tallyline_reference_data::api::{ReferenceApplication, ReferenceDataCache};
use tallyline_shared_kernel::api::Clock;
use tallyline_shared_kernel::api::model::{CommodityId, Timestamp};

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
            Some(120),
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
        Some(120),
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
            Some(120),
        )]))
        .unwrap();
    assert_eq!(
        application.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(7_200))
    );
    assert_eq!(application.commodities(UexEnvironment::Live, "").len(), 1);
    assert!(application.due().is_empty());
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
            Commodity::new(CommodityId(7), "Test Ore", "SHARED", false, Some(82)),
            Commodity::new(CommodityId(8), "Live Ore", "SHARED", true, Some(120)),
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
    assert!(
        items
            .iter()
            .all(|item| item.average_price(UexEnvironment::Ptu).is_none())
    );
    assert_eq!(
        items.first().unwrap().average_price(UexEnvironment::Live),
        Some(82)
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
        EndpointData::Commodities(vec![Commodity::new(CommodityId(1), "Old", "A", true, None)]),
    );
    let mut application = ReferenceApplication::load(MemoryCache(vec![old]), FixedClock).unwrap();
    application.refresh_failed(Endpoint::Commodities);
    application
        .refreshed(EndpointData::Commodities(vec![Commodity::new(
            CommodityId(2),
            "New",
            "B",
            true,
            None,
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
