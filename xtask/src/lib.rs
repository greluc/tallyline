// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Repository checks and generators for the Tallyline workspace.

#![forbid(unsafe_code)]

pub mod architecture;
pub mod build_code;
pub mod check;
pub mod dco;
pub mod deny;
pub mod dependabot;
pub mod english;
pub mod exit;
pub mod guard;
pub mod hashlist;
pub mod header;
pub mod licence;
pub mod listing;
pub mod lockfile;
pub mod native;
pub mod native_assembly;
pub mod notices;
pub mod osv;
pub mod quality;
pub mod register;
pub mod registry;
pub mod requirement;
pub mod sources;
pub mod sqlite_native;
pub mod tags;
pub mod target_graph;
pub mod tracked;
pub mod vendor;
pub mod vet;

#[cfg(test)]
mod denygate;
#[cfg(test)]
mod negative;
#[cfg(test)]
mod scratch;
#[cfg(test)]
mod token_reconciliation;

#[cfg(test)]
mod quality_tests;

#[cfg(test)]
mod native_scope_tests;

#[cfg(test)]
mod build_code_tests;

#[cfg(test)]
mod native_assembly_tests;

#[cfg(test)]
mod sqlite_native_tests;

#[cfg(test)]
mod build_code_workspace_tests;

mod ort_scope;

#[cfg(test)]
mod target_graph_tests;
