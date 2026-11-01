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
