// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Versioned application settings and owner-only application directories.

#![forbid(unsafe_code)]

pub mod api;
mod config;
mod game_config;
mod game_log;
mod game_log_mapping;
mod game_log_parser;
mod layout;
mod log_time;
mod logo;
mod witness_output;

mod update_config;

mod backup;
mod image_intake;
mod manual_attachment;

mod private_artifact;
