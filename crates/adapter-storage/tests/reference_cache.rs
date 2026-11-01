// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reference cache contracts against real temporary SQLite files.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tallyline_adapter_storage::api::SqliteReferenceCache;
use tallyline_reference_data::api::ReferenceDataCache;
use tallyline_reference_data::api::model::{CachedReference, Commodity, EndpointData};
use tallyline_shared_kernel::api::model::{CommodityId, Timestamp};

struct TestDirectory(PathBuf);
impl TestDirectory {
    fn new() -> std::io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "tallyline-reference-db-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for TestDirectory {
    fn drop(&mut self) {
        assert!(
            std::fs::remove_dir_all(&self.0).is_ok(),
            "temporary database cleanup failed"
        );
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_nf_11__empty_database_migrates_and_cached_data_survives_reopening() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("tallyline.db");
    let entry = CachedReference::new(
        Timestamp::from_seconds(1_800),
        EndpointData::Commodities(vec![Commodity::new(
            CommodityId(42),
            "Agricium",
            "AGRI",
            true,
            Some(120),
        )]),
    );
    {
        let mut cache = SqliteReferenceCache::open(&path).unwrap();
        assert!(cache.load().unwrap().is_empty());
        cache.replace(&entry).unwrap();
    }
    let mut reopened = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(reopened.load().unwrap(), vec![entry]);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__cached_fetch_times_keep_millisecond_precision_and_full_second_range() {
    let directory = TestDirectory::new().unwrap();
    let mut cache = SqliteReferenceCache::open(directory.0.join("tallyline.db")).unwrap();
    for fetched_at in [
        Timestamp::from_millis(1_701_234_567_891),
        Timestamp::from_seconds(u64::MAX),
    ] {
        let entry = CachedReference::new(fetched_at, EndpointData::Commodities(vec![]));
        cache.replace(&entry).unwrap();
        assert_eq!(cache.load().unwrap(), vec![entry]);
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11__future_database_is_refused_without_modifying_its_bytes() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("future.db");
    {
        let database = rusqlite::Connection::open(&path).unwrap();
        database.pragma_update(None, "user_version", 2).unwrap();
    }
    let before = std::fs::read(&path).unwrap();
    assert!(matches!(
        SqliteReferenceCache::open(&path),
        Err(
            tallyline_adapter_storage::api::StorageError::UnsupportedSchema {
                found: 2,
                supported: 1
            }
        )
    ));
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__failed_transaction_preserves_previous_content_and_successful_fetch_time() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("tallyline.db");
    let old = CachedReference::new(
        Timestamp::from_seconds(1_800),
        EndpointData::Commodities(vec![Commodity::new(
            CommodityId(1),
            "Old",
            "A",
            true,
            Some(120),
        )]),
    );
    {
        let mut cache = SqliteReferenceCache::open(&path).unwrap();
        cache.replace(&old).unwrap();
        let duplicate = CachedReference::new(
            Timestamp::from_seconds(3_600),
            EndpointData::Commodities(vec![
                Commodity::new(CommodityId(2), "First", "B", true, None),
                Commodity::new(CommodityId(2), "Second", "C", true, None),
            ]),
        );
        assert!(cache.replace(&duplicate).is_err());
        assert_eq!(cache.load().unwrap(), vec![old.clone()]);
    }
    let mut reopened = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(reopened.load().unwrap(), vec![old]);
}
