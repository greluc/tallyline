// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Application preferences acceptance tests.

use tallyline_workflows::api::model::{AppSettings, Theme};
use tallyline_workflows::api::{AppSettingsStore, SettingsApplication};

#[derive(Default)]
struct MemorySettings(Option<AppSettings>);

impl AppSettingsStore for MemorySettings {
    type Error = std::convert::Infallible;

    fn save(&mut self, settings: &AppSettings) -> Result<(), Self::Error> {
        self.0 = Some(settings.clone());
        Ok(())
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_7_q_theme_r_nf_5__theme_changes_are_saved_before_becoming_active() {
    let mut application =
        SettingsApplication::new(AppSettings::default(), MemorySettings::default());
    application.change_theme(Theme::HighContrast).unwrap();
    assert_eq!(application.settings().theme(), Theme::HighContrast);
    assert_eq!(
        application.into_store().0.unwrap().theme(),
        Theme::HighContrast
    );
}

struct FailedSettings;

impl AppSettingsStore for FailedSettings {
    type Error = &'static str;

    fn save(&mut self, _: &AppSettings) -> Result<(), Self::Error> {
        Err("directory is read-only")
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_5__failed_save_preserves_the_active_theme() {
    let mut application = SettingsApplication::new(AppSettings::default(), FailedSettings);
    assert_eq!(
        application.change_theme(Theme::Light),
        Err("directory is read-only")
    );
    assert_eq!(application.settings().theme(), Theme::System);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_19_q_app__onboarding_completion_is_persisted_before_leaving_the_start_screen() {
    let mut application =
        SettingsApplication::new(AppSettings::default(), MemorySettings::default());
    assert!(!application.settings().onboarding_complete());
    application.complete_onboarding().unwrap();
    assert!(application.settings().onboarding_complete());
    assert!(application.into_store().0.unwrap().onboarding_complete());
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_7_q_theme__new_preferences_follow_the_system_theme() {
    assert_eq!(AppSettings::default().theme(), Theme::System);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_7__release_versions_follow_numeric_stable_tag_order() {
    use tallyline_workflows::api::model::ReleaseVersion;
    assert!(ReleaseVersion::parse("v2.0.0").unwrap() > ReleaseVersion::parse("v1.99.99").unwrap());
    assert!(ReleaseVersion::parse("v1.10.0").unwrap() > ReleaseVersion::parse("v1.9.9").unwrap());
    assert_eq!(
        ReleaseVersion::parse("v2.0.1").unwrap().release_url(),
        "https://github.com/greluc/tallyline/releases/tag/v2.0.1"
    );
    assert_eq!(
        ReleaseVersion::parse("v0.0.0").unwrap().to_string(),
        "v0.0.0"
    );
    for tag in [
        "v1.0.0-rc.1",
        "v1.0.0+build",
        "1.0.0",
        "v01.0.0",
        "v1.2",
        "v1.2.3.4",
        "v1.2.x",
        "v+1.2.3",
        "v1..3",
        "v1.2.",
        "v18446744073709551616.0.0",
    ] {
        assert!(ReleaseVersion::parse(tag).is_none(), "{tag}");
    }
}
