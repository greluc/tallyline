// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Durable numeric review through real SQLite repositories.

use tallyline_adapter_storage::api::SqliteReferenceCache;
use tallyline_game::api::model::GameEnvironment;
use tallyline_reference_data::api::model::{
    CachedReference, EndpointData, PricePrior, PriorProvenance, PriorValues, TerminalPriors,
    UexEnvironment,
};
use tallyline_reference_data::api::{ReferenceApplication, ReferenceDataCache};
use tallyline_reporting::api::model::ReportKey;
use tallyline_reporting::api::{ManualEdit, ReportRepository, ReportingApplication};
use tallyline_shared_kernel::api::Clock;
use tallyline_shared_kernel::api::model::{CommodityId, TerminalId, Timestamp, TradeSide};
struct Fixed;
impl Clock for Fixed {
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
fn reference(
    price: &str,
) -> Result<tallyline_reference_data::api::model::ReferenceSnapshot, Box<dyn std::error::Error>> {
    let prior = PricePrior::new(
        CommodityId(42),
        TradeSide::Buy,
        PriorValues {
            price: price.parse()?,
            stock: "0".parse()?,
            status: std::num::NonZeroU8::new(3),
            container_sizes: vec![],
        },
        PriorProvenance {
            game_version: "4.10.2".to_owned(),
            modified_at: Timestamp::from_seconds(900),
        },
    )?;
    Ok(ReferenceApplication::load(
        Cache(vec![CachedReference::new(
            Timestamp::from_seconds(950),
            EndpointData::TerminalPriors(TerminalPriors::new(TerminalId(11), vec![prior])?),
        )]),
        Fixed,
    )?
    .snapshot(UexEnvironment::Live))
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11_r_ui_12_q_manual_r_val_7_q_manual__price_approval_pending_revocation_and_focus_survive_reopening()
 {
    let path =
        std::env::temp_dir().join(format!("tallyline-price-review-{}.db", std::process::id()));
    let mut app = ReportingApplication::new(SqliteReferenceCache::open(&path).unwrap(), Fixed);
    let key = ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap();
    let report = app.create(key, Some("4.10.2")).unwrap();
    let report = app
        .edit(
            report.id(),
            report.version(),
            ManualEdit::AddCommodity(CommodityId(42)),
            &reference("100").unwrap(),
        )
        .unwrap();
    let row = report.rows()[0].id();
    let report = app
        .edit(
            report.id(),
            report.version(),
            ManualEdit::Price(row, Some("100".parse().unwrap())),
            &reference("100").unwrap(),
        )
        .unwrap();
    let approved = app
        .confirm_price(
            report.id(),
            report.version(),
            row,
            &reference("100").unwrap(),
        )
        .unwrap();
    drop(app);
    let mut store = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(
        store.load_draft(approved.id()).unwrap(),
        Some(approved.clone())
    );
    let mut app = ReportingApplication::new(store, Fixed);
    let revoked = app
        .reassess_prices(approved.id(), approved.version(), &reference("90").unwrap())
        .unwrap();
    assert!(revoked.rows()[0].price_confirmation_revoked());
    assert!(revoked.rows()[0].price_unreviewed());
    drop(app);
    let mut store = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(
        store.load_draft(revoked.id()).unwrap(),
        Some(revoked.clone())
    );
    let mut app = ReportingApplication::new(store, Fixed);
    let focused = app
        .focus_price(revoked.id(), revoked.version(), row)
        .unwrap();
    assert!(!focused.rows()[0].price_unreviewed());
    assert!(focused.rows()[0].price_needs_confirmation());
    let confirmed = app
        .confirm_price(
            focused.id(),
            focused.version(),
            row,
            &reference("90").unwrap(),
        )
        .unwrap();
    assert!(!confirmed.rows()[0].price_confirmation_revoked());
    assert!(!confirmed.rows()[0].price_needs_confirmation());
    drop(app);
    let mut store = SqliteReferenceCache::open(&path).unwrap();
    assert_eq!(store.load_draft(confirmed.id()).unwrap(), Some(confirmed));
    drop(store);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11_r_ui_12_q_manual__contradictory_persisted_price_decisions_are_refused() {
    use tallyline_reporting::api::RepositoryError;
    let path =
        std::env::temp_dir().join(format!("tallyline-price-corrupt-{}.db", std::process::id()));
    let mut store = SqliteReferenceCache::open(&path).unwrap();
    let mut report = store
        .create_manual(
            ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
            Timestamp::from_seconds(900),
            None,
        )
        .unwrap();
    let row = report.add_commodity(CommodityId(42)).unwrap();
    report.set_price(row, Some("100".parse().unwrap())).unwrap();
    store.save_draft(&report, 0).unwrap();
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    let payload: String = connection
        .query_row("SELECT payload FROM reports", [], |row| row.get(0))
        .unwrap();
    let original: serde_json::Value = serde_json::from_str(&payload).unwrap();
    for (price, approval, deviation, unreviewed, revoked) in [
        (
            serde_json::Value::Null,
            serde_json::json!(["100", "equal", "live"]),
            serde_json::json!("equal"),
            false,
            false,
        ),
        (
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::json!("minor"),
            false,
            false,
        ),
        (
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::Value::Null,
            true,
            false,
        ),
        (
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::Value::Null,
            false,
            true,
        ),
        (
            serde_json::json!("100"),
            serde_json::json!(["101", "major", "live"]),
            serde_json::json!("major"),
            false,
            false,
        ),
        (
            serde_json::json!("100"),
            serde_json::json!(["100", "minor", "live"]),
            serde_json::json!("major"),
            false,
            false,
        ),
        (
            serde_json::json!("100"),
            serde_json::json!(["100", "major", "live"]),
            serde_json::Value::Null,
            false,
            false,
        ),
        (
            serde_json::json!("100"),
            serde_json::json!(["100", "major", "live"]),
            serde_json::json!("major"),
            true,
            false,
        ),
        (
            serde_json::json!("100"),
            serde_json::json!(["100", "major", "live"]),
            serde_json::json!("major"),
            false,
            true,
        ),
        (
            serde_json::json!("100"),
            serde_json::Value::Null,
            serde_json::Value::Null,
            true,
            false,
        ),
        (
            serde_json::json!("100"),
            serde_json::Value::Null,
            serde_json::Value::Null,
            false,
            true,
        ),
        (
            serde_json::json!("100"),
            serde_json::Value::Null,
            serde_json::json!("minor"),
            true,
            false,
        ),
    ] {
        let mut document = original.clone();
        let row = &mut document["rows"][0];
        row["price"] = price;
        row["price_decision"]["confirmation"] = approval;
        row["price_decision"]["deviation"] = deviation;
        row["price_decision"]["unreviewed"] = serde_json::json!(unreviewed);
        row["price_decision"]["revoked"] = serde_json::json!(revoked);
        connection
            .execute("UPDATE reports SET payload=?1", [document.to_string()])
            .unwrap();
        let mut store = SqliteReferenceCache::open(&path).unwrap();
        assert_eq!(
            store.load_draft(report.id()),
            Err(RepositoryError::Invalid),
            "{document}"
        );
        assert_eq!(store.drafts(), Err(RepositoryError::Invalid));
    }
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_11_r_ui_12_q_manual__schema_six_requires_backup_and_preserves_legacy_price_and_status_state()
 {
    use tallyline_adapter_storage::api::StorageError;
    let path = std::env::temp_dir().join(format!(
        "tallyline-price-migrate-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let backup = path.with_extension("backup.db");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TABLE reports(id INTEGER PRIMARY KEY AUTOINCREMENT,version INTEGER NOT NULL,payload TEXT NOT NULL); CREATE TABLE report_recents(report_id INTEGER PRIMARY KEY,changed_order INTEGER NOT NULL); CREATE TABLE screenshot_requirements(account_digest BLOB PRIMARY KEY NOT NULL CHECK(length(account_digest)=32),observed_at TEXT NOT NULL); PRAGMA user_version=6;").unwrap();
    let payload = r#"{"terminal":11,"side":"buy","environment":"live","observed_at":"900123","game_version":"4.10.2","rows":[{"id":1,"commodity":42,"price":"123.456","stock":"17","status":3,"status_confirmation":[3,"major","live"],"status_deviation":"minor","status_unreviewed":false,"status_confirmation_revoked":false}],"attachment":null}"#;
    let mut document: serde_json::Value = serde_json::from_str(payload).unwrap();
    document["attachment"] = serde_json::json!({"image": vec![1;32], "rectangle": [10,20,30,40], "observed_at": "900123", "time_source": "user", "preview_confirmed": true, "working_copy": vec![2;32]});
    let payload = document.to_string();
    use sha2::{Digest, Sha256};
    let account = Sha256::digest(b"migration-account");
    connection
        .execute(
            "INSERT INTO screenshot_requirements VALUES(?1,'777000')",
            [account.as_slice()],
        )
        .unwrap();
    connection
        .execute("INSERT INTO reports VALUES(1,14,?1)", [&payload])
        .unwrap();
    connection
        .execute("INSERT INTO report_recents VALUES(1,9)", [])
        .unwrap();
    drop(connection);
    assert!(matches!(
        SqliteReferenceCache::open(&path),
        Err(StorageError::MigrationBackupRequired)
    ));
    let mut store = SqliteReferenceCache::open_with_migration_backup(&path, |cache, version| {
        assert_eq!(version, 6);
        cache.backup_into(&backup, Timestamp::from_seconds(1000), None)?;
        Ok(())
    })
    .unwrap();
    let report = store.drafts().unwrap().remove(0);
    use tallyline_submission::api::ScreenshotPolicyStore;
    assert_eq!(
        store.required_since("migration-account").unwrap(),
        Some(Timestamp::from_millis(777000))
    );
    assert_eq!(store.required_since("another-account").unwrap(), None);
    let attachment = report.attachment().unwrap();
    assert!(attachment.preview_confirmed());
    assert_eq!(
        attachment.working_copy(),
        Some(tallyline_reporting::api::model::ImageHash::new([2; 32]))
    );
    assert_eq!(
        attachment.binding(),
        tallyline_reporting::api::model::AttachmentBinding::new(
            tallyline_reporting::api::model::ImageHash::new([1; 32]),
            tallyline_reporting::api::model::AttachmentRegion::new(10, 20, 30, 40).unwrap()
        )
    );
    assert_eq!(report.version(), 14);
    assert_eq!(report.observed_at(), Timestamp::from_millis(900123));
    assert_eq!(report.game_version(), Some("4.10.2"));
    assert_eq!(report.rows()[0].price().unwrap().to_string(), "123.456");
    assert_eq!(report.rows()[0].stock().unwrap().to_string(), "17");
    assert_eq!(
        report.rows()[0]
            .status_confirmation()
            .unwrap()
            .value()
            .get(),
        3
    );
    assert!(report.rows()[0].price_confirmation().is_none());
    assert!(report.rows()[0].price_deviation().is_none());
    assert!(!report.rows()[0].price_unreviewed());
    assert!(!report.rows()[0].price_confirmation_revoked());
    let before = rusqlite::Connection::open(&backup).unwrap();
    assert_eq!(
        before
            .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        6
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
        7
    );
    assert_eq!(
        current
            .query_row("SELECT changed_order FROM report_recents", [], |row| row
                .get::<_, u32>(0))
            .unwrap(),
        9
    );
    drop(current);
    drop(before);
    drop(store);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(backup).unwrap();
}
