// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! SQLite persistence, forward migrations and atomic reference cache repositories.

#![forbid(unsafe_code)]

pub mod api;
mod reference_cache;
