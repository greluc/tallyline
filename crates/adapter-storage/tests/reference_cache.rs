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

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_ui_8__terminals_and_commodities_reopen_with_independent_fetch_times() {
    use tallyline_reference_data::api::model::{Terminal, TerminalText};
    use tallyline_shared_kernel::api::model::TerminalId;
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("tallyline.db");
    let commodity = CachedReference::new(
        Timestamp::from_millis(10_001),
        EndpointData::Commodities(vec![]),
    );
    let terminals = CachedReference::new(
        Timestamp::from_millis(90_999),
        EndpointData::Terminals(vec![Terminal::new(
            TerminalId(17),
            TerminalText {
                name: "Trade Centre".to_owned(),
                nickname: "TDD".to_owned(),
                display_name: "Trade Centre".to_owned(),
                location: "Stanton / ArcCorp / Area18".to_owned(),
            },
            false,
        )]),
    );
    {
        let mut cache = SqliteReferenceCache::open(&path).unwrap();
        cache.replace(&commodity).unwrap();
        cache.replace(&terminals).unwrap();
    }
    let mut cache = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(cache.load().unwrap(), vec![commodity, terminals]);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_cap_3b__version_cache_roundtrip_preserves_non_semver_live_and_absent_ptu() {
    use tallyline_reference_data::api::model::GameVersions;
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("tallyline.db");
    for ptu in [Some("3.24.2a".to_owned()), None] {
        let entry = CachedReference::new(
            Timestamp::from_millis(90_999),
            EndpointData::GameVersions(GameVersions {
                live: "4.9".to_owned(),
                ptu,
            }),
        );
        {
            let mut cache = SqliteReferenceCache::open(&path).unwrap();
            cache.replace(&entry).unwrap();
        }
        let mut reopened = SqliteReferenceCache::open(&path).unwrap();
        assert_eq!(reopened.load().unwrap(), vec![entry]);
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11__sqlite_backup_includes_uncheckpointed_wal_and_a_restore_marker() {
    use tallyline_adapter_storage::api::model::BackupMetadata;
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("tallyline.db");
    let mut cache = SqliteReferenceCache::open(&path).unwrap();
    let writer = rusqlite::Connection::open(&path).unwrap();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    writer.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
    let entry = CachedReference::new(
        Timestamp::from_millis(90_999),
        EndpointData::Commodities(vec![Commodity::new(
            CommodityId(7),
            "WAL record",
            "WAL",
            true,
            None,
        )]),
    );
    cache.replace(&entry).unwrap();
    assert!(
        std::fs::metadata(path.with_extension("db-wal"))
            .unwrap()
            .len()
            > 0
    );
    let destination = directory.0.join("before.db");
    let taken_at = Timestamp::from_millis(123_456_789);
    assert_eq!(
        cache.backup_into(&destination, taken_at, Some(3)).unwrap(),
        1
    );
    let mut backup = SqliteReferenceCache::open(&destination).unwrap();
    assert_eq!(backup.load().unwrap(), vec![entry.clone()]);
    assert_eq!(
        backup.backup_metadata().unwrap(),
        Some(BackupMetadata {
            taken_at,
            from_schema: 1,
            configuration_schema: Some(3)
        })
    );
    assert_eq!(cache.backup_metadata().unwrap(), None);
    assert_eq!(cache.load().unwrap(), vec![entry]);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11__ambiguous_restore_markers_fail_closed() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("tallyline.db");
    let cache = SqliteReferenceCache::open(&path).unwrap();
    let writer = rusqlite::Connection::open(&path).unwrap();
    writer.execute_batch("CREATE TABLE backup_meta(taken_at TEXT NOT NULL,from_version INTEGER NOT NULL,config_schema INTEGER); INSERT INTO backup_meta VALUES('123456',1,3),('999999',1,3);").unwrap();
    assert!(matches!(
        cache.backup_metadata(),
        Err(tallyline_adapter_storage::api::StorageError::InvalidBackup)
    ));
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11__backup_never_replaces_a_file_and_keeps_absent_config_and_exact_time() {
    let directory = TestDirectory::new().unwrap();
    let mut cache = SqliteReferenceCache::open(directory.0.join("tallyline.db")).unwrap();
    let destination = directory.0.join("existing.db");
    std::fs::write(&destination, b"keep existing bytes").unwrap();
    assert!(matches!(
        cache.backup_into(&destination, Timestamp::from_seconds(10), None),
        Err(tallyline_adapter_storage::api::StorageError::Filesystem(_))
    ));
    assert_eq!(std::fs::read(&destination).unwrap(), b"keep existing bytes");
    let fresh = directory.0.join("new.db");
    let taken_at = Timestamp::from_seconds(u64::MAX);
    cache.backup_into(&fresh, taken_at, None).unwrap();
    let backup = SqliteReferenceCache::open(&fresh).unwrap();
    let metadata = backup.backup_metadata().unwrap().unwrap();
    assert_eq!(metadata.taken_at, taken_at);
    assert_eq!(metadata.configuration_schema, None);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__status_vocabulary_reopens_without_collapsing_buy_and_sell_labels() {
    use std::num::NonZeroU8;
    use tallyline_reference_data::api::model::{StatusLevel, StatusLevels};
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("tallyline.db");
    let entry = CachedReference::new(
        Timestamp::from_seconds(90),
        EndpointData::StatusLevels(StatusLevels {
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
        }),
    );
    {
        let mut cache = SqliteReferenceCache::open(&path).unwrap();
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
fn r_api_5__invalid_status_replacement_preserves_the_last_successful_snapshot() {
    use std::num::NonZeroU8;
    use tallyline_reference_data::api::model::{StatusLevel, StatusLevels};
    let directory = TestDirectory::new().unwrap();
    let mut cache = SqliteReferenceCache::open(directory.0.join("tallyline.db")).unwrap();
    let level = StatusLevel {
        code: NonZeroU8::new(1).unwrap(),
        name: "Out of Stock".to_owned(),
        short_name: "Empty".to_owned(),
        abbreviation: "OS".to_owned(),
    };
    let old = CachedReference::new(
        Timestamp::from_seconds(100),
        EndpointData::StatusLevels(StatusLevels {
            buy: vec![level.clone()],
            sell: vec![],
        }),
    );
    cache.replace(&old).unwrap();
    let invalid = CachedReference::new(
        Timestamp::from_seconds(200),
        EndpointData::StatusLevels(StatusLevels {
            buy: vec![level.clone(), level],
            sell: vec![],
        }),
    );
    assert!(matches!(
        cache.replace(&invalid),
        Err(tallyline_adapter_storage::api::StorageError::InvalidCache)
    ));
    assert_eq!(cache.load().unwrap(), vec![old]);
}
