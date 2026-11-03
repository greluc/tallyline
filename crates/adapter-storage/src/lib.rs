// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! SQLite persistence, consistent backups, forward migrations and reference repositories.

#![forbid(unsafe_code)]

pub mod api;
mod backup;
mod migration;
mod parameter_document;
mod prior_document;
mod reference_cache;
mod reports;
mod status_document;

mod screenshot_policy;

mod report_lifecycle;

mod queue_document;

mod queue;

mod accepted_priors;

mod receipt_enrichment;

mod manual_dispatch;

mod event_outbox;

mod reference_consistency;
