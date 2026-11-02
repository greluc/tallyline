// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Application composition and coordinated local-storage bootstrap.

#![forbid(unsafe_code)]

pub mod api;
mod attachment_worker;
mod cpu_worker;
mod executor;
mod manual_worker;
mod reference_worker;
mod storage_bootstrap;

mod manual_attachments;

mod credential_worker;

mod credential_logs;
mod manual_reviews;

mod build_info;

mod build_entry;
