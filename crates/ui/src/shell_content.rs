// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Existing page contents and command gathering.

use crate::api::model::{SettingsCommand, UserAction, View};
use crate::api::{Catalogue, FanKitUnit, ShellState};
use tallyline_workflows::api::model::Theme;

pub(crate) fn show(
    ui: &mut egui::Ui,
    state: &mut ShellState,
    catalogue: &Catalogue,
    unit: &FanKitUnit,
) -> Vec<SettingsCommand> {
    let mut commands = Vec::new();
    egui::ScrollArea::vertical()
        .animated(false)
        .id_salt("shell-page-scroll")
        .show(ui, |ui| match state.view() {
            View::Onboarding => {
                if let Some(action) = crate::api::show_onboarding_view(ui, state, catalogue, unit) {
                    commands.extend(state.act(action));
                }
                crate::credentials::show(ui, state.credentials_mut(), catalogue);
                ui.label(catalogue.text("release.onboarding_test"));
                if ui
                    .add_enabled(
                        state.ready(),
                        egui::Button::new(tallyline_egui::button_text(
                            catalogue.text("release.prepare_test"),
                        )),
                    )
                    .clicked()
                {
                    state.show_manual();
                }
            }
            View::Queue => {
                if let Some(feedback) = &state.submission.feedback {
                    crate::submission_feedback::show(ui, feedback, catalogue);
                }
                let theme =
                    crate::api::resolve_theme(state.settings().theme(), ui.ctx().system_theme());
                let command = ui
                    .add_enabled_ui(!state.submission_busy(), |ui| {
                        crate::api::show_submission_queue(
                            ui,
                            &mut state.submission.queue,
                            catalogue,
                            theme,
                        )
                    })
                    .inner;
                if let Some(command) = command {
                    state.queue_cancel_intent(command);
                }
            }
            View::Manual => crate::manual_views::show_manual_editor(ui, state, catalogue),
            View::Reference => crate::reference_views::show_browser_contents(ui, state, catalogue),
            View::Home => {
                ui.heading(catalogue.text("home.title"));
                ui.label(catalogue.text("home.empty"));
            }
            View::Settings => {
                ui.heading(catalogue.text("settings.title"));
                ui.label(catalogue.text("settings.theme"));
                for (theme, key) in [
                    (Theme::System, "settings.system"),
                    (Theme::Dark, "settings.dark"),
                    (Theme::Light, "settings.light"),
                    (Theme::HighContrast, "settings.high_contrast"),
                ] {
                    if ui
                        .add_enabled(
                            state.ready() && !state.busy(),
                            egui::RadioButton::new(
                                state.settings().theme() == theme,
                                catalogue.text(key),
                            ),
                        )
                        .clicked()
                    {
                        commands.extend(state.act(UserAction::ChooseTheme(theme)));
                    }
                }
                let stock = egui::CollapsingHeader::new(catalogue.text("settings.stock_title"))
                    .show(ui, |ui| crate::stock_settings::show(ui, state, catalogue));
                commands.extend(stock.body_returned.unwrap_or_default());
                let submission = egui::CollapsingHeader::new(catalogue.text("settings.submission"))
                    .show(ui, |ui| {
                        crate::api::show_submission_settings(ui, state, catalogue)
                    });
                commands.extend(submission.body_returned.unwrap_or_default());
                commands.extend(crate::updates::settings(ui, state, catalogue));
                commands.extend(crate::reference_views::show_mapping(ui, state, catalogue));
                commands.extend(crate::game_witness::show(ui, state, catalogue));
                if ui
                    .button(tallyline_egui::button_text(catalogue.text("action.done")))
                    .clicked()
                {
                    state.leave_settings();
                }
                egui::CollapsingHeader::new(catalogue.text("credentials.title")).show(ui, |ui| {
                    crate::credentials::show(ui, state.credentials_mut(), catalogue);
                });
            }
        });
    commands
}
