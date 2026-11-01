// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Durable manual Draft repository contracts.

use tallyline_adapter_storage::api::SqliteReferenceCache;
use tallyline_game::api::model::GameEnvironment;
use tallyline_reporting::api::model::ReportKey;
use tallyline_reporting::api::{ReportRepository, RepositoryError};
use tallyline_shared_kernel::api::model::{CommodityId, TerminalId, Timestamp, TradeSide};

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11_r_man_2__entered_draft_survives_reopening_and_rejects_stale_writes() {
    let path = std::env::temp_dir().join(format!(
        "tallyline-report-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut store = SqliteReferenceCache::open(&path).unwrap();
    let mut report = store
        .create_manual(
            ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
            Timestamp::from_millis(100123),
            Some("4.10.2"),
        )
        .unwrap();
    let row = report.add_commodity(CommodityId(42)).unwrap();
    report
        .set_price(row, Some("123.456".parse().unwrap()))
        .unwrap();
    report.set_stock(row, Some("0".parse().unwrap())).unwrap();
    report.set_status(row, std::num::NonZeroU8::new(3)).unwrap();
    store.save_draft(&report, 0).unwrap();
    assert!(matches!(
        store.save_draft(&report, 0),
        Err(RepositoryError::Conflict)
    ));
    drop(store);
    let mut reopened = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(
        reopened.load_draft(report.id()).unwrap(),
        Some(report.clone())
    );
    assert_eq!(reopened.drafts().unwrap(), vec![report]);
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11_r_man_2__schema_two_requires_a_verified_backup_before_drafts_are_available() {
    let path = std::env::temp_dir().join(format!(
        "tallyline-report-migration-{}.db",
        std::process::id()
    ));
    {
        let database = rusqlite::Connection::open(&path).unwrap();
        database.execute_batch("CREATE TABLE ref_cache(endpoint TEXT PRIMARY KEY,fetched_at TEXT NOT NULL,payload TEXT NOT NULL); CREATE TABLE ref_commodities(id INTEGER PRIMARY KEY,name TEXT NOT NULL,code TEXT NOT NULL,available_live INTEGER NOT NULL); CREATE TABLE ref_terminals(id INTEGER PRIMARY KEY,name TEXT NOT NULL,nickname TEXT NOT NULL,display_name TEXT NOT NULL,location TEXT NOT NULL,available_live INTEGER NOT NULL,max_container_size INTEGER); PRAGMA user_version=2;").unwrap();
    }
    assert!(matches!(
        SqliteReferenceCache::open(&path),
        Err(tallyline_adapter_storage::api::StorageError::MigrationBackupRequired)
    ));
    let backup = path.with_extension("backup.db");
    let mut store = SqliteReferenceCache::open_with_migration_backup(&path, |cache, from| {
        assert_eq!(from, 2);
        cache.backup_into(&backup, Timestamp::from_seconds(100), Some(4))?;
        Ok(())
    })
    .unwrap();
    assert!(store.drafts().unwrap().is_empty());
    let mut restored =
        SqliteReferenceCache::open_with_migration_backup(&backup, |_, _| Ok(())).unwrap();
    assert_eq!(restored.backup_metadata().unwrap().unwrap().from_schema, 2);
    assert!(restored.drafts().unwrap().is_empty());
    drop(restored);
    drop(store);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(backup).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11_r_man_2__explicit_takeover_requires_a_matching_prior_and_preserves_its_exact_values() {
    use tallyline_reference_data::api::model::{
        CachedReference, EndpointData, PricePrior, PriorProvenance, PriorValues, TerminalPriors,
        UexEnvironment,
    };
    use tallyline_reference_data::api::{ReferenceApplication, ReferenceDataCache};
    use tallyline_reporting::api::{ManualEdit, ReportingApplication};
    struct Fixed;
    impl tallyline_shared_kernel::api::Clock for Fixed {
        fn now(&self) -> Timestamp {
            Timestamp::from_seconds(1000)
        }
    }
    struct Cache(Vec<CachedReference>);
    impl ReferenceDataCache for Cache {
        type Error = std::convert::Infallible;
        fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
            Ok(self.0.clone())
        }
        fn replace(&mut self, _: &CachedReference) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    let prior = PricePrior::new(
        CommodityId(42),
        TradeSide::Buy,
        PriorValues {
            price: "123.45".parse().unwrap(),
            stock: "0".parse().unwrap(),
            status: std::num::NonZeroU8::new(3),
            container_sizes: vec![],
        },
        PriorProvenance {
            game_version: "4.10.2".to_owned(),
            modified_at: Timestamp::from_seconds(900),
        },
    )
    .unwrap();
    let reference = ReferenceApplication::load(
        Cache(vec![CachedReference::new(
            Timestamp::from_seconds(950),
            EndpointData::TerminalPriors(TerminalPriors::new(TerminalId(11), vec![prior]).unwrap()),
        )]),
        Fixed,
    )
    .unwrap();
    let snapshot = reference.snapshot(UexEnvironment::Live);
    let path = std::env::temp_dir().join(format!(
        "tallyline-report-takeover-{}.db",
        std::process::id()
    ));
    let mut application =
        ReportingApplication::new(SqliteReferenceCache::open(&path).unwrap(), Fixed);
    let key = ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap();
    let mut report = application.create(key, None).unwrap();
    report = application
        .edit(
            report.id(),
            0,
            ManualEdit::AddCommodity(CommodityId(42)),
            &snapshot,
        )
        .unwrap();
    assert!(!report.rows().first().unwrap().entered());
    let row = report.rows().first().unwrap().id();
    report = application
        .edit(
            report.id(),
            report.version(),
            ManualEdit::TakeOver(row),
            &snapshot,
        )
        .unwrap();
    assert_eq!(
        report.rows().first().unwrap().price().unwrap().to_string(),
        "123.45"
    );
    assert_eq!(
        report.rows().first().unwrap().stock().unwrap().to_string(),
        "0"
    );
    assert!(
        application
            .edit(
                report.id(),
                report.version(),
                ManualEdit::TakeOver(row),
                &reference.snapshot(UexEnvironment::Ptu)
            )
            .is_err()
    );
    drop(application);
    std::fs::remove_file(path).unwrap();
}
