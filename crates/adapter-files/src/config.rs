// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Atomic, versioned application preferences.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use tallyline_workflows::api::AppSettingsStore;
use tallyline_workflows::api::model::{AppSettings, Theme};

/// Loaded preferences and visible validation failures.
#[derive(Debug)]
pub struct LoadedSettings {
    /// Valid preferences, with invalid values replaced by defaults.
    pub settings: AppSettings,
    /// Settings that require the user's attention.
    pub errors: Vec<SettingsIssue>,
}

/// A rejected persisted preference.
#[derive(Debug, PartialEq, Eq)]
pub struct SettingsIssue {
    /// The invalid setting's catalogue key.
    pub key: &'static str,
    /// The rejected non-secret value.
    pub rejected: String,
    /// The default used instead.
    pub default: &'static str,
}

/// The single writer of a versioned settings file.
pub struct ConfigFile {
    path: PathBuf,
}

impl ConfigFile {
    /// Creates a store at its resolved configuration path.
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Reads versioned preferences and reports rejected values.
    ///
    /// # Errors
    /// Returns filesystem failures or an unsupported future schema version.
    pub fn load(&self) -> io::Result<LoadedSettings> {
        let content = match fs::read(&self.path) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(LoadedSettings {
                    settings: AppSettings::default(),
                    errors: Vec::new(),
                });
            }
            Err(error) => return Err(error),
        };
        let document: serde_json::Value = match serde_json::from_slice(&content) {
            Ok(document) => document,
            Err(_) => {
                return Ok(LoadedSettings {
                    settings: AppSettings::default(),
                    errors: vec![SettingsIssue {
                        key: "settings.document",
                        rejected: "invalid JSON".to_owned(),
                        default: "application defaults",
                    }],
                });
            }
        };
        if !matches!(
            document
                .get("schema_version")
                .and_then(serde_json::Value::as_u64),
            Some(1 | 2)
        ) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported settings schema version",
            ));
        }
        let mut errors = Vec::new();
        let theme = match document
            .get("app")
            .and_then(|app| app.get("theme"))
            .and_then(serde_json::Value::as_str)
        {
            Some("system") => Theme::System,
            Some("dark") => Theme::Dark,
            Some("light") => Theme::Light,
            Some("high_contrast") => Theme::HighContrast,
            _ => {
                errors.push(SettingsIssue {
                    key: "settings.theme",
                    rejected: document
                        .get("app")
                        .and_then(|app| app.get("theme"))
                        .map_or_else(|| "missing".to_owned(), serde_json::Value::to_string),
                    default: "system",
                });
                Theme::System
            }
        };
        let onboarding = match document
            .get("app")
            .and_then(|app| app.get("onboarding_complete"))
            .and_then(serde_json::Value::as_bool)
        {
            Some(value) => value,
            None => {
                errors.push(SettingsIssue {
                    key: "settings.onboarding_complete",
                    rejected: document
                        .get("app")
                        .and_then(|app| app.get("onboarding_complete"))
                        .map_or_else(|| "missing".to_owned(), serde_json::Value::to_string),
                    default: "false",
                });
                false
            }
        };
        Ok(LoadedSettings {
            settings: AppSettings::from_persisted(theme, onboarding),
            errors,
        })
    }
}

impl AppSettingsStore for ConfigFile {
    type Error = io::Error;
    fn save(&mut self, settings: &AppSettings) -> io::Result<()> {
        let theme = match settings.theme() {
            Theme::System => "system",
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::HighContrast => "high_contrast",
        };
        let mut document = self.read_document()?;
        Self::object(&mut document)?.insert(
            "app".to_owned(),
            serde_json::json!({"theme":theme,"onboarding_complete":settings.onboarding_complete()}),
        );
        self.write_document(document)
    }
}

impl ConfigFile {
    pub(crate) fn object(
        document: &mut serde_json::Value,
    ) -> io::Result<&mut serde_json::Map<String, serde_json::Value>> {
        document.as_object_mut().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "settings document is not an object",
            )
        })
    }
    pub(crate) fn read_document(&self) -> io::Result<serde_json::Value> {
        let document: serde_json::Value = match fs::read(&self.path) {
            Ok(content) => serde_json::from_slice(&content)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid settings JSON"))?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                serde_json::json!({"schema_version":2})
            }
            Err(error) => return Err(error),
        };
        if !matches!(
            document
                .get("schema_version")
                .and_then(serde_json::Value::as_u64),
            Some(1 | 2)
        ) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported settings schema version",
            ));
        }
        Ok(document)
    }

    pub(crate) fn write_document(&mut self, mut document: serde_json::Value) -> io::Result<()> {
        Self::object(&mut document)?.insert("schema_version".to_owned(), serde_json::json!(2));
        let content = serde_json::to_vec_pretty(&document)?;
        let temporary = self.path.with_extension("json.pending");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let result = (|| {
            file.write_all(&content)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, &self.path)
        })();
        if result.is_err() {
            fs::remove_file(&temporary)?;
        }
        result
    }
}
