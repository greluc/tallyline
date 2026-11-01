// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Tallyline desktop shell, legal unit and user-driven view commands.

#![forbid(unsafe_code)]

mod dialog_presenter;
mod state;

mod catalogue;
mod fan_kit;
mod fonts;

mod shell;

mod activation;
pub mod api;
mod desktop;
mod game_witness;
mod legal_views;
mod licences;
mod theme;

mod updates;

mod reference_browser;

mod reference_views;
