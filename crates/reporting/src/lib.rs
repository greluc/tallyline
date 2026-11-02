// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reporting owns manual Draft commands and persistence contracts.

#![forbid(unsafe_code)]

pub mod api;
mod application;
mod assessment;
mod attachment;
mod confirmation;
mod price_assessment;
mod report;

mod manual_review;
mod price_confirmation;
