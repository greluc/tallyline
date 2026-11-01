// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The complete legal placements shared by desktop rendering and pinning tests.

use crate::api::model::{LegalPlacement, UserAction};
use crate::api::{Catalogue, FanKitUnit, ShellState};

/// A direct user request from the About content.
pub enum AboutAction {
    /// Show the full bundled GPL text.
    ShowGpl,
    /// Show the generated offline component catalogue.
    ShowLicences,
}

/// Draws the first onboarding screen and returns its direct user action.
pub fn show_onboarding_view(
    ui: &mut egui::Ui,
    state: &ShellState,
    catalogue: &Catalogue,
    unit: &FanKitUnit,
) -> Option<UserAction> {
    unit.show_placement(ui, LegalPlacement::Onboarding, catalogue);
    ui.label(catalogue.text("onboarding.introduction"));
    ui.add_enabled(
        state.ready() && !state.busy(),
        egui::Button::new(catalogue.text("onboarding.continue")),
    )
    .clicked()
    .then_some(UserAction::ContinueOnboarding)
}

/// Draws all About content and returns direct user requests without opening a dialog.
pub fn show_about_view(
    ui: &mut egui::Ui,
    catalogue: &Catalogue,
    unit: &FanKitUnit,
) -> Vec<AboutAction> {
    let mut actions = Vec::new();
    unit.show_placement(ui, LegalPlacement::About, catalogue);
    ui.label(catalogue.text("about.unofficial"));
    ui.label(catalogue.text("about.copyright"));
    ui.label(format!(
        "{} · {}/{}",
        catalogue.text("about.development"),
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    ui.label(catalogue.text("about.warranty"));
    if ui.button(catalogue.text("about.gpl")).clicked() {
        actions.push(AboutAction::ShowGpl);
    }
    if ui.button(catalogue.text("about.licences")).clicked() {
        actions.push(AboutAction::ShowLicences);
    }
    ui.hyperlink_to(
        catalogue.text("about.source"),
        "https://github.com/greluc/tallyline",
    );
    ui.hyperlink_to(
        catalogue.text("about.release"),
        "https://github.com/greluc/tallyline/releases",
    );
    actions
}
