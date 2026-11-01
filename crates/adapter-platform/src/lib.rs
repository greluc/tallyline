// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! OS instance election and bounded local activation without network ports.

#![deny(unsafe_code)]

pub mod api;
#[cfg(test)]
mod contract_tests;
mod instance;
mod protocol;
mod socket;
#[cfg(windows)]
mod windows_socket;
