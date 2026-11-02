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
    report = application
        .confirm_status(report.id(), report.version(), row, &snapshot)
        .unwrap();
    assert!(report.rows()[0].status_confirmation().is_some());
    drop(application);
    let mut reopened = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(
        reopened.load_draft(report.id()).unwrap(),
        Some(report.clone())
    );
    assert_eq!(
        report.rows()[0]
            .status_confirmation()
            .unwrap()
            .reference_environment(),
        UexEnvironment::Live
    );
    let expected = report.version();
    report.set_status(row, std::num::NonZeroU8::new(5)).unwrap();
    reopened.save_draft(&report, expected).unwrap();
    let mut application = ReportingApplication::new(reopened, Fixed);
    report = application
        .reassess_statuses(report.id(), report.version(), &snapshot)
        .unwrap();
    assert!(report.rows()[0].status_unreviewed());
    assert!(report.rows()[0].status_needs_confirmation());
    drop(application);
    let mut reopened = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(
        reopened.load_draft(report.id()).unwrap(),
        Some(report.clone())
    );
    let mut application = ReportingApplication::new(reopened, Fixed);
    report = application
        .confirm_status(report.id(), report.version(), row, &snapshot)
        .unwrap();
    application.set_mapping(
        tallyline_game::api::model::EnvironmentMapping::default()
            .with_environment(GameEnvironment::Live, UexEnvironment::Ptu),
    );
    report = application
        .reassess_statuses(
            report.id(),
            report.version(),
            &reference.snapshot(UexEnvironment::Ptu),
        )
        .unwrap();
    assert!(report.rows()[0].status_confirmation_revoked());
    drop(application);
    let mut reopened = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(
        reopened.load_draft(report.id()).unwrap(),
        Some(report.clone())
    );
    let mut application = ReportingApplication::new(reopened, Fixed);
    report = application
        .focus_status(report.id(), report.version(), row)
        .unwrap();
    assert!(!report.rows()[0].status_unreviewed());
    assert!(report.rows()[0].status_needs_confirmation());
    drop(application);
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
fn r_nf_11_r_man_2_r_man_5_r_val_6_q_manual__corrupt_rows_and_attachment_times_are_refused_during_real_database_reconstitution()
 {
    let path = std::env::temp_dir().join(format!(
        "tallyline-report-corrupt-{}.db",
        std::process::id()
    ));
    let mut store = SqliteReferenceCache::open(&path).unwrap();
    let mut report = store
        .create_manual(
            ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
            Timestamp::from_seconds(100),
            Some("4.10.2"),
        )
        .unwrap();
    report.add_commodity(CommodityId(42)).unwrap();
    store.save_draft(&report, 0).unwrap();
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    let payload: String = connection
        .query_row("SELECT payload FROM reports", [], |row| row.get(0))
        .unwrap();
    let document: serde_json::Value = serde_json::from_str(&payload).unwrap();
    let row = document["rows"][0].clone();
    let mut invalid_rows = Vec::new();
    for (field, value) in [
        ("id", serde_json::json!(0)),
        ("commodity", serde_json::json!(0)),
        ("price", serde_json::json!("0")),
    ] {
        let mut invalid = row.clone();
        invalid[field] = value;
        invalid_rows.push(vec![invalid]);
    }
    let mut duplicate_identity = row.clone();
    duplicate_identity["commodity"] = serde_json::json!(43);
    invalid_rows.push(vec![row.clone(), duplicate_identity]);
    let mut duplicate_commodity = row.clone();
    duplicate_commodity["id"] = serde_json::json!(2);
    invalid_rows.push(vec![row.clone(), duplicate_commodity]);
    let mut invalid = row.clone();
    invalid["status_confirmation"] = serde_json::json!([3, "major", "live"]);
    invalid_rows.push(vec![invalid]);
    for (confirmation, deviation, unreviewed) in [
        (
            serde_json::json!([3, "major", "live"]),
            serde_json::json!("major"),
            false,
        ),
        (
            serde_json::json!([4, "major", "live"]),
            serde_json::json!("major"),
            false,
        ),
        (
            serde_json::json!([3, "major", "live"]),
            serde_json::json!("major"),
            true,
        ),
        (serde_json::Value::Null, serde_json::json!("major"), false),
        (serde_json::Value::Null, serde_json::Value::Null, true),
    ] {
        let mut invalid = row.clone();
        invalid["status_confirmation"] = confirmation;
        invalid["status_deviation"] = deviation;
        invalid["status_unreviewed"] = serde_json::json!(unreviewed);
        invalid_rows.push(vec![invalid]);
    }
    for (confirmation, deviation, unreviewed) in [
        (
            serde_json::json!([3, "major", "live"]),
            serde_json::Value::Null,
            false,
        ),
        (
            serde_json::json!([3, "minor", "live"]),
            serde_json::json!("major"),
            false,
        ),
        (serde_json::Value::Null, serde_json::Value::Null, true),
    ] {
        let mut invalid = row.clone();
        invalid["status"] = serde_json::json!(3);
        invalid["status_confirmation"] = confirmation;
        invalid["status_deviation"] = deviation;
        invalid["status_unreviewed"] = serde_json::json!(unreviewed);
        invalid_rows.push(vec![invalid]);
    }
    for (status, confirmation, deviation) in [
        (
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::Value::Null,
        ),
        (
            serde_json::json!(3),
            serde_json::json!([3, "major", "live"]),
            serde_json::json!("major"),
        ),
        (
            serde_json::json!(3),
            serde_json::Value::Null,
            serde_json::Value::Null,
        ),
    ] {
        let mut invalid = row.clone();
        invalid["status"] = status;
        invalid["status_confirmation"] = confirmation;
        invalid["status_deviation"] = deviation;
        invalid["status_confirmation_revoked"] = serde_json::json!(true);
        invalid_rows.push(vec![invalid]);
    }
    let mut invalid = row.clone();
    invalid["status"] = serde_json::json!(3);
    invalid["status_deviation"] = serde_json::json!("minor");
    invalid["status_unreviewed"] = serde_json::json!(true);
    invalid_rows.push(vec![invalid]);
    for rows in invalid_rows {
        let mut invalid = document.clone();
        invalid["rows"] = serde_json::json!(rows);
        connection
            .execute("UPDATE reports SET payload=?1", [invalid.to_string()])
            .unwrap();
        let mut reopened = SqliteReferenceCache::open(&path).unwrap();
        assert_eq!(
            reopened.load_draft(report.id()),
            Err(RepositoryError::Invalid)
        );
        assert_eq!(reopened.drafts(), Err(RepositoryError::Invalid));
    }
    for source in ["imported_file", "user"] {
        let mut invalid = document.clone();
        invalid["attachment"] = serde_json::json!({
            "image": vec![1; 32],
            "rectangle": [10,20,30,40],
            "observed_at": "90000",
            "time_source": source,
            "preview_confirmed": true
        });
        connection
            .execute("UPDATE reports SET payload=?1", [invalid.to_string()])
            .unwrap();
        let mut reopened = SqliteReferenceCache::open(&path).unwrap();
        assert_eq!(
            reopened.load_draft(report.id()),
            Err(RepositoryError::Invalid)
        );
        assert_eq!(reopened.drafts(), Err(RepositoryError::Invalid));
    }
    connection
        .execute("UPDATE reports SET payload=?1", [payload])
        .unwrap();
    let mut reopened = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(reopened.load_draft(report.id()).unwrap(), Some(report));
    drop(reopened);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11_r_ui_12_q_manual__schema_three_requires_backup_and_preserves_legacy_user_values() {
    use tallyline_adapter_storage::api::StorageError;
    let path = std::env::temp_dir().join(format!(
        "tallyline-status-migration-{}.db",
        std::process::id()
    ));
    let backup = path.with_extension("backup.db");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TABLE reports(id INTEGER PRIMARY KEY AUTOINCREMENT,version INTEGER NOT NULL,payload TEXT NOT NULL); CREATE TABLE report_recents(report_id INTEGER PRIMARY KEY,changed_order INTEGER NOT NULL); PRAGMA user_version=3;").unwrap();
    let payload = r#"{"terminal":11,"side":"buy","environment":"live","observed_at":"100123","game_version":"4.10.2","rows":[{"id":1,"commodity":42,"price":"123.456","stock":"0","status":3}]}"#;
    connection
        .execute("INSERT INTO reports VALUES(1,4,?1)", [payload])
        .unwrap();
    connection
        .execute("INSERT INTO report_recents VALUES(1,1)", [])
        .unwrap();
    drop(connection);
    assert!(matches!(
        SqliteReferenceCache::open(&path),
        Err(StorageError::MigrationBackupRequired)
    ));
    let mut store = SqliteReferenceCache::open_with_migration_backup(&path, |cache, from| {
        assert_eq!(from, 3);
        cache.backup_into(&backup, Timestamp::from_seconds(100), None)?;
        Ok(())
    })
    .unwrap();
    let report = store.drafts().unwrap().remove(0);
    assert_eq!(report.version(), 4);
    assert_eq!(report.observed_at(), Timestamp::from_millis(100123));
    assert_eq!(report.rows()[0].price().unwrap().to_string(), "123.456");
    assert_eq!(report.rows()[0].status().unwrap().get(), 3);
    assert!(report.rows()[0].status_confirmation().is_none());
    assert!(!report.rows()[0].status_unreviewed());
    let before = rusqlite::Connection::open(&backup).unwrap();
    assert_eq!(
        before
            .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        before
            .query_row("SELECT payload FROM reports", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        payload
    );
    let current = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        current
            .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        5
    );
    drop(current);
    drop(before);
    drop(store);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(backup).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_5_r_sub_7_r_val_6__attachment_confirmation_is_durable_and_bound_to_geometry_and_time() {
    use tallyline_reporting::api::model::{
        AttachmentBinding, AttachmentRegion, AttachmentTime, ImageHash,
    };
    use tallyline_reporting::api::{AttachmentEdit, ReportingApplication};
    use tallyline_shared_kernel::api::Clock;
    struct Fixed;
    impl Clock for Fixed {
        fn now(&self) -> Timestamp {
            Timestamp::from_seconds(1000)
        }
    }
    let path = std::env::temp_dir().join(format!("tallyline-attachment-{}.db", std::process::id()));
    let mut app = ReportingApplication::new(SqliteReferenceCache::open(&path).unwrap(), Fixed);
    let report = app
        .create(
            ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
            Some("4.10.2"),
        )
        .unwrap();
    let binding = AttachmentBinding::new(
        ImageHash::new([1; 32]),
        AttachmentRegion::new(10, 20, 30, 40).unwrap(),
    );
    let report = app
        .edit_attachment(
            report.id(),
            report.version(),
            AttachmentEdit::Select(
                binding,
                AttachmentTime::ImportedFile(Timestamp::from_seconds(900)),
            ),
        )
        .unwrap();
    assert!(!report.attachment().unwrap().preview_confirmed());
    assert!(report.attachment_gate().is_err());
    let report = app
        .edit_attachment(
            report.id(),
            report.version(),
            AttachmentEdit::ConfirmPreview(binding),
        )
        .unwrap();
    assert!(report.attachment().unwrap().preview_confirmed());
    assert!(report.attachment_gate().is_err());
    let report = app
        .edit_attachment(
            report.id(),
            report.version(),
            AttachmentEdit::ConfirmTime(Timestamp::from_seconds(899)),
        )
        .unwrap();
    assert!(report.attachment_gate().is_ok());
    assert_eq!(report.observed_at(), Timestamp::from_seconds(899));
    drop(app);
    let mut app = ReportingApplication::new(SqliteReferenceCache::open(&path).unwrap(), Fixed);
    let report = app
        .drafts()
        .unwrap()
        .into_iter()
        .find(|r| r.id() == report.id())
        .unwrap();
    assert!(report.attachment_gate().is_ok());
    let changed = AttachmentBinding::new(
        ImageHash::new([1; 32]),
        AttachmentRegion::new(10, 21, 30, 40).unwrap(),
    );
    let report = app
        .edit_attachment(
            report.id(),
            report.version(),
            AttachmentEdit::Select(
                changed,
                AttachmentTime::ImportedFile(Timestamp::from_seconds(950)),
            ),
        )
        .unwrap();
    assert!(!report.attachment().unwrap().preview_confirmed());
    assert_eq!(report.observed_at(), Timestamp::from_seconds(899));
    assert!(
        app.edit_attachment(
            report.id(),
            report.version(),
            AttachmentEdit::ConfirmPreview(binding)
        )
        .is_err()
    );
    assert!(
        app.edit_attachment(
            report.id(),
            report.version(),
            AttachmentEdit::ConfirmTime(Timestamp::from_seconds(951))
        )
        .is_err()
    );
    let report = app
        .edit_attachment(report.id(), report.version(), AttachmentEdit::Remove)
        .unwrap();
    assert_eq!(report.observed_at(), Timestamp::from_seconds(899));
    assert!(report.attachment().is_none());
    drop(app);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11_r_man_5__schema_four_migrates_without_inventing_attachment_or_changing_report_decisions()
{
    use tallyline_adapter_storage::api::StorageError;
    let directory = std::env::temp_dir().join(format!(
        "tallyline-attachment-migration-{}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("application.db");
    let backup = directory.join("before.db");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TABLE reports(id INTEGER PRIMARY KEY AUTOINCREMENT,version INTEGER NOT NULL,payload TEXT NOT NULL); CREATE TABLE report_recents(report_id INTEGER PRIMARY KEY,changed_order INTEGER NOT NULL); PRAGMA user_version=4;").unwrap();
    let payload = r#"{"terminal":11,"side":"buy","environment":"live","observed_at":"100123","game_version":"4.10.2","rows":[{"id":1,"commodity":42,"price":"123.456","stock":"0","status":3,"status_confirmation":[3,"major","live"],"status_deviation":"major","status_unreviewed":false,"status_confirmation_revoked":false}]}"#;
    connection
        .execute("INSERT INTO reports VALUES(1,7,?1)", [payload])
        .unwrap();
    connection
        .execute("INSERT INTO report_recents VALUES(1,1)", [])
        .unwrap();
    drop(connection);
    assert!(matches!(
        SqliteReferenceCache::open(&path),
        Err(StorageError::MigrationBackupRequired)
    ));
    let mut store = SqliteReferenceCache::open_with_migration_backup(&path, |cache, from| {
        assert_eq!(from, 4);
        cache.backup_into(&backup, Timestamp::from_seconds(100), None)?;
        Ok(())
    })
    .unwrap();
    let report = store
        .load_draft(tallyline_reporting::api::model::ReportId(1))
        .unwrap()
        .unwrap();
    assert!(report.attachment().is_none());
    assert_eq!(report.version(), 7);
    assert_eq!(report.observed_at(), Timestamp::from_millis(100123));
    assert!(
        report
            .rows()
            .first()
            .unwrap()
            .status_confirmation()
            .is_some()
    );
    let before = rusqlite::Connection::open(&backup).unwrap();
    assert_eq!(
        before
            .query_row("SELECT payload FROM reports", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        payload
    );
    drop(before);
    drop(store);
    std::fs::remove_dir_all(directory).unwrap();
}
