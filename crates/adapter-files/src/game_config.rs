// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Local witness preferences in the shared configuration file.

use crate::config::{ConfigFile, SettingsIssue};
use std::io;
use tallyline_game::api::GameSettingsStore;
use tallyline_game::api::model::{GameSettings, WitnessMode};

/// Loaded witness preferences with a local-only channel selection.
pub struct LoadedGamePreferences {
    /// Valid game preferences, defaulting to no file access.
    pub settings: GameSettings,
    /// User-selected channel directory, never included in witness output.
    pub channel_directory: Option<String>,
    /// Safe validation issues without rejected raw source values.
    pub errors: Vec<SettingsIssue>,
}

impl ConfigFile {
    /// Loads game preferences, treating version-one files as default off.
    ///
    /// # Errors
    /// Returns read failures or unsupported configuration schemas.
    pub fn load_game_preferences(&self) -> io::Result<LoadedGamePreferences> {
        let document = self.read_document()?;
        let game = document.get("game");
        let mut errors = Vec::new();
        let witness = match game.and_then(|game| game.get("log_witness")) {
            Some(serde_json::Value::Bool(true)) => WitnessMode::On,
            Some(serde_json::Value::Bool(false)) | None => WitnessMode::Off,
            Some(_) => {
                errors.push(SettingsIssue {
                    key: "settings.log_witness",
                    rejected: "invalid boolean".to_owned(),
                    default: "off",
                });
                WitnessMode::Off
            }
        };
        let channel_directory = match game.and_then(|game| game.get("channel_directory")) {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(path)) => {
                if path.is_empty() {
                    None
                } else {
                    Some(path.clone())
                }
            }
            Some(_) => {
                errors.push(SettingsIssue {
                    key: "settings.game_channel",
                    rejected: "invalid directory value".to_owned(),
                    default: "no channel",
                });
                None
            }
        };
        Ok(LoadedGamePreferences {
            settings: GameSettings { witness },
            channel_directory,
            errors,
        })
    }

    /// Changes the selected channel and atomically disables the witness.
    ///
    /// # Errors
    /// Returns a configuration or persistence failure without replacing preferences.
    pub fn change_game_channel(&mut self, channel: &str) -> io::Result<()> {
        let mut document = self.read_document()?;
        Self::object(&mut document)?.insert(
            "game".to_owned(),
            serde_json::json!({"channel_directory":channel,"log_witness":false}),
        );
        self.write_document(document)
    }
}
impl GameSettingsStore for ConfigFile {
    type Error = io::Error;
    fn save_game_settings(&mut self, settings: &GameSettings) -> io::Result<()> {
        let mut document = self.read_document()?;
        if !document
            .get("game")
            .is_some_and(serde_json::Value::is_object)
        {
            Self::object(&mut document)?.insert("game".to_owned(), serde_json::json!({}));
        }
        let game = document.get_mut("game").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "game preferences are missing")
        })?;
        Self::object(game)?.insert(
            "log_witness".to_owned(),
            serde_json::json!(settings.witness == WitnessMode::On),
        );
        self.write_document(document)
    }
}
