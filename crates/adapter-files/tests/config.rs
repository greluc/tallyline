// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! ConfigFile contract tests against actual files.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tallyline_adapter_files::api::ConfigFile;
use tallyline_workflows::api::AppSettingsStore;
use tallyline_workflows::api::model::{AppSettings, Theme};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> std::io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "tallyline-config-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory)?;
        Ok(Self(directory))
    }
}

impl Drop for TestDirectory {
    #[expect(
        clippy::unwrap_used,
        reason = "Cleanup failure must fail the real-filesystem contract test."
    )]
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_5__settings_survive_reopening_and_atomic_replacement() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("settings.json");
    let mut store = ConfigFile::new(path.clone());
    let initial = store.load().unwrap();
    assert_eq!(initial.settings, AppSettings::default());
    assert!(initial.errors.is_empty());
    store
        .save(&AppSettings::from_persisted(Theme::Light, false))
        .unwrap();
    store
        .save(&AppSettings::from_persisted(Theme::HighContrast, true))
        .unwrap();
    let reopened = ConfigFile::new(path).load().unwrap();
    assert_eq!(
        reopened.settings,
        AppSettings::from_persisted(Theme::HighContrast, true)
    );
    assert!(reopened.errors.is_empty());
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 1);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_5__invalid_settings_report_rejected_value_and_default() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("settings.json");
    std::fs::write(
        &path,
        br#"{"schema_version":1,"app":{"theme":42,"onboarding_complete":false}}"#,
    )
    .unwrap();
    let loaded = ConfigFile::new(path).load().unwrap();
    assert_eq!(loaded.settings.theme(), Theme::System);
    assert_eq!(loaded.errors.len(), 1);
    let issue = loaded.errors.first().unwrap();
    assert_eq!(issue.key, "settings.theme");
    assert_eq!(issue.rejected, "42");
    assert_eq!(issue.default, "system");
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_2_q_portable_r_nf_5__portable_storage_stays_under_data_and_failed_self_tests_are_reported()
{
    let directory = TestDirectory::new().unwrap();
    let layout = tallyline_adapter_files::api::StorageLayout::portable(&directory.0);
    assert_eq!(layout.config(), directory.0.join("data/config"));
    assert_eq!(layout.data(), directory.0.join("data/data"));
    assert_eq!(layout.logs(), directory.0.join("data/logs"));
    assert_eq!(layout.cache(), directory.0.join("data/cache"));
    assert!(layout.self_test().is_empty());
    assert!(layout.config().is_dir());
    std::fs::remove_dir(layout.logs()).unwrap();
    std::fs::write(layout.logs(), b"blocked").unwrap();
    let failures = layout.self_test();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures.first().unwrap().path, layout.logs());
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_5__installed_layout_uses_only_profile_and_xdg_roots() {
    let directory = TestDirectory::new().unwrap();
    let roaming = directory.0.join("roaming");
    let local = directory.0.join("local");
    let windows = tallyline_adapter_files::api::StorageLayout::installed_windows(&roaming, &local);
    assert_eq!(windows.config(), roaming.join("Tallyline"));
    assert_eq!(windows.data(), local.join("Tallyline/data"));
    assert_eq!(windows.logs(), local.join("Tallyline/logs"));
    assert_eq!(windows.cache(), local.join("Tallyline/cache"));
    let linux = tallyline_adapter_files::api::StorageLayout::installed_linux(
        &roaming,
        &local,
        &directory.0.join("state"),
        &directory.0.join("cache"),
    );
    assert_eq!(linux.config(), roaming.join("tallyline"));
    assert_eq!(linux.data(), local.join("tallyline"));
    assert_eq!(linux.logs(), directory.0.join("state/tallyline/logs"));
    assert_eq!(linux.cache(), directory.0.join("cache/tallyline"));
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_5__saving_app_preferences_preserves_other_context_sections() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("settings.json");
    std::fs::write(&path, br#"{"schema_version":1,"capture":{"folders":[]},"app":{"theme":"dark","onboarding_complete":false}}"#).unwrap();
    ConfigFile::new(path.clone())
        .save(&AppSettings::from_persisted(Theme::Light, false))
        .unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        saved.get("capture"),
        Some(&serde_json::json!({"folders": []}))
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_5__future_schema_is_never_overwritten() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("settings.json");
    let original = br#"{"schema_version":99,"app":{"theme":"dark","onboarding_complete":false}}"#;
    std::fs::write(&path, original).unwrap();
    let mut store = ConfigFile::new(path.clone());
    assert!(store.load().is_err());
    assert!(store.save(&AppSettings::default()).is_err());
    assert_eq!(std::fs::read(path).unwrap(), original);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_5__installed_data_and_logs_match_the_architecture_storage_rows() {
    use tallyline_adapter_files::api::StorageLayout;
    let windows = StorageLayout::installed_windows(
        std::path::Path::new("roaming"),
        std::path::Path::new("local"),
    );
    let linux = StorageLayout::installed_linux(
        std::path::Path::new("config"),
        std::path::Path::new("data"),
        std::path::Path::new("state"),
        std::path::Path::new("cache"),
    );
    assert_eq!(
        (windows.data(), linux.logs()),
        (
            std::path::Path::new("local/Tallyline/data"),
            std::path::Path::new("state/tallyline/logs")
        )
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_5__invalid_onboarding_reports_only_the_rejected_setting_and_default() {
    let directory = TestDirectory::new().unwrap();
    let path = directory.0.join("settings.json");
    for rejected in [
        serde_json::json!("true"),
        serde_json::json!(42),
        serde_json::Value::Null,
    ] {
        let document = serde_json::json!({"schema_version":1,"app":{"theme":"dark","onboarding_complete":rejected},"account":{"token":"private-marker"}});
        std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        let loaded = ConfigFile::new(path.clone()).load().unwrap();
        assert!(!loaded.settings.onboarding_complete());
        assert_eq!(loaded.errors.len(), 1);
        let issue = loaded.errors.first().unwrap();
        assert_eq!(issue.key, "settings.onboarding_complete");
        assert_eq!(issue.rejected, rejected.to_string());
        assert_eq!(issue.default, "false");
        assert!(!issue.rejected.contains("private-marker"));
    }
}
