// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! OS instance election, bounded local activation and optional release metadata checks.

#![deny(unsafe_code)]

pub mod api;
#[cfg(test)]
mod contract_tests;
mod instance;
mod protocol;
mod socket;
#[cfg(windows)]
mod windows_socket;

mod release_token;

mod ocr_startup;
mod releases;
