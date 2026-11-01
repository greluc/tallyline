// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Repository checks and generators for the Tallyline workspace.

#![forbid(unsafe_code)]

pub mod check;
pub mod corpus;
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
pub mod notices;
pub mod osv;
pub mod register;
pub mod registry;
pub mod requirement;
pub mod sources;
pub mod tags;
pub mod tracked;
pub mod vendor;
pub mod vet;

#[cfg(test)]
mod denygate;
#[cfg(test)]
mod negative;
#[cfg(test)]
mod scratch;
