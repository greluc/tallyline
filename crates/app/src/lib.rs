// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Application composition and coordinated local-storage bootstrap.

#![forbid(unsafe_code)]

pub mod api;
mod executor;
mod reference_worker;
mod storage_bootstrap;
