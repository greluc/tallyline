// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Event-driven desktop views and commands.

use crate::api::model::{SettingsCommand, UserAction, View};
use crate::api::{Catalogue, FanKitUnit, ShellState};
use tallyline_workflows::api::model::Theme;

/// Draws the shell from owned state and returns user-requested worker commands.
pub fn show_shell(
    ui: &mut egui::Ui,
    state: &mut ShellState,
    catalogue: &Catalogue,
    unit: &FanKitUnit,
) -> Vec<SettingsCommand> {
    let mut commands = Vec::new();
    egui::MenuBar::new().ui(ui, |ui| {
        if ui
            .add_enabled(
                state.settings().onboarding_complete(),
                egui::Button::new(catalogue.text("menu.home")),
            )
            .clicked()
        {
            state.leave_settings();
        }
        if ui.button(catalogue.text("reference.title")).clicked() {
            state.show_reference();
        }
        if ui.button(catalogue.text("menu.settings")).clicked() {
            state.show_settings();
        }
        if ui.button(catalogue.text("manual.title")).clicked() {
            state.show_manual();
        }
        ui.menu_button(catalogue.text("menu.help"), |ui| {
            if ui.button(catalogue.text("menu.about")).clicked() {
                state.open_about();
                ui.close();
            }
        });
    });
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        for key in ["status.mode", "status.queue"] {
            ui.label(catalogue.text(key));
            ui.separator();
        }
    });
    crate::reference_views::show_reference_status(ui, state, catalogue);
    commands.extend(crate::updates::notice(ui, state, catalogue));
    if let Some(notice) = state.notice() {
        ui.label(catalogue.text("status.action_required"));
        ui.label(notice);
    }
    if !state.ready() && state.notice().is_none() {
        ui.label(catalogue.text("status.starting"));
    }
    if state.busy() {
        ui.label(catalogue.text("status.saving"));
    }
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| match state.view() {
        View::Onboarding => {
            if let Some(action) = crate::api::show_onboarding_view(ui, state, catalogue, unit) {
                commands.extend(state.act(action));
            }
            crate::credentials::show(ui, state.credentials_mut(), catalogue);
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
            commands.extend(crate::updates::settings(ui, state, catalogue));
            commands.extend(crate::reference_views::show_mapping(ui, state, catalogue));
            commands.extend(crate::game_witness::show(ui, state, catalogue));
            if ui.button(catalogue.text("action.done")).clicked() {
                state.leave_settings();
            }
            egui::CollapsingHeader::new(catalogue.text("credentials.title")).show(ui, |ui| {
                crate::credentials::show(ui, state.credentials_mut(), catalogue);
            });
        }
    });
    state.show_dialogs(ui.ctx(), catalogue, unit);
    commands
}

/// Wakes the UI once after a channel result arrives.
pub fn wake_up(context: &egui::Context) {
    context.request_repaint();
}
