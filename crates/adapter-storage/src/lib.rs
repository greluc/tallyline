// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! SQLite persistence, consistent backups, forward migrations and reference repositories.

#![forbid(unsafe_code)]

pub mod api;
mod backup;
mod migration;
mod parameter_document;
mod reference_cache;
mod status_document;
