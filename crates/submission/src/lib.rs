// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Account validation and secure credential persistence for submission.
//!
//! Secret-bearing public values cannot be debug-logged (R-NF-4).
//!
//! ```compile_fail,E0277
//! use tallyline_submission::api::model::ReleaseToken;
//! fn log(value: &ReleaseToken) { let _ = format!("{value:?}"); }
//! ```
//!
//! ```compile_fail,E0277
//! use tallyline_submission::api::model::CredentialValues;
//! fn log(value: &CredentialValues) { let _ = format!("{value:?}"); }
//! ```
//!
//! ```compile_fail,E0277
//! use tallyline_submission::api::model::CredentialCandidate;
//! fn log(value: &CredentialCandidate) { let _ = format!("{value:?}"); }
//! ```
//!
//! ```compile_fail,E0277
//! use tallyline_submission::api::model::SavedCredentials;
//! fn log(value: &SavedCredentials) { let _ = format!("{value:?}"); }
//! ```
//!
//! ```compile_fail,E0277
//! use tallyline_submission::api::ValidationRequest;
//! fn log(value: &ValidationRequest) { let _ = format!("{value:?}"); }
//! ```

#![forbid(unsafe_code)]

pub mod api;

mod screenshot_policy;

mod application;
mod budget;
mod events;
mod job;
mod preflight;
mod receipt;
mod request;
mod schedule;
mod server_time;

mod receipt_mapping;

mod observation_order;

mod queue_wait;
