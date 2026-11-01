// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The UEX API adapter: a read client and `data_submit` dry-run client for UEX API 2.0.

#![forbid(unsafe_code)]

pub mod api;
#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod contract_tests;
mod envelope;
mod parameter_wire;
mod prior_wire;
mod reference_fetcher;
mod reference_wire;
mod transport;
mod wire;

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
mod terminal_contract_tests;

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
mod proxy_contract_tests;
#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
mod status_contract_tests;

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
mod parameter_contract_tests;

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
mod prior_contract_tests;
