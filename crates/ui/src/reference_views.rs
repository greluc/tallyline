// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cached reference browsing and environment status presentation.

use crate::api::{Catalogue, ShellState};
use tallyline_game::api::model::GameEnvironment;
use tallyline_reference_data::api::model::{Endpoint, ReferenceState, UexEnvironment};

pub(crate) const CHANNELS: [(GameEnvironment, &str); 5] = [
    (GameEnvironment::Live, "reference.live"),
    (GameEnvironment::Hotfix, "reference.hotfix"),
    (GameEnvironment::Ptu, "reference.ptu"),
    (GameEnvironment::Eptu, "reference.eptu"),
    (GameEnvironment::TechPreview, "reference.tech_preview"),
];

/// Draws cached vocabularies and queues only explicit reference requests.
pub fn show_reference_browser(ui: &mut egui::Ui, state: &mut ShellState, catalogue: &Catalogue) {
    show_reference_status(ui, state, catalogue);
    show_browser_contents(ui, state, catalogue);
}

pub(crate) fn show_browser_contents(
    ui: &mut egui::Ui,
    state: &mut ShellState,
    catalogue: &Catalogue,
) {
    ui.heading(catalogue.text("reference.title"));
    ui.label(catalogue.text("reference.channel"));
    ui.horizontal_wrapped(|ui| {
        for (channel, key) in CHANNELS {
            if ui
                .selectable_label(state.references().channel() == channel, catalogue.text(key))
                .clicked()
            {
                state.references_mut().choose_channel(channel);
            }
        }
    });
    if ui
        .add_enabled(
            !state.references().refreshing(),
            egui::Button::new(catalogue.text("reference.refresh")),
        )
        .clicked()
    {
        state.references_mut().refresh();
    }
    if let Some(snapshot) = state.reference_snapshot()
        && let Some(versions) = snapshot.game_versions()
    {
        let version = match snapshot.environment() {
            UexEnvironment::Live => Some(versions.live.as_str()),
            UexEnvironment::Ptu => versions.ptu.as_deref(),
        };
        ui.label(format!(
            "{} {}",
            catalogue.text("reference.version"),
            version.unwrap_or(catalogue.text("reference.no_version"))
        ));
    }
    ui.separator();
    terminals(ui, state, catalogue);
    ui.separator();
    commodities(ui, state, catalogue);
}

pub(crate) fn show_reference_status(ui: &mut egui::Ui, state: &ShellState, catalogue: &Catalogue) {
    let mapping = &state.game_witness().settings.environment_mapping;
    let environment = mapping.environment(state.references().channel());
    ui.label(format!(
        "{} {}",
        catalogue.text("reference.environment"),
        environment_name(environment, catalogue)
    ));
    let key = match state.reference_snapshot().map(|snapshot| snapshot.state()) {
        None | Some(ReferenceState::NoReferenceData) => "reference.no_data",
        Some(ReferenceState::Cached) => "reference.cached",
        Some(ReferenceState::Offline) => "reference.offline",
    };
    ui.label(catalogue.text(key));
    if state.references().refreshing() {
        ui.label(catalogue.text("reference.refreshing"));
    }
    if let Some(snapshot) = state.reference_snapshot() {
        ui.horizontal_wrapped(|ui| {
            for (endpoint, key) in [
                (Endpoint::Terminals, "reference.terminals"),
                (Endpoint::Commodities, "reference.commodities"),
                (Endpoint::GameVersions, "reference.versions"),
                (Endpoint::StatusLevels, "reference.status_levels"),
                (Endpoint::Parameters, "reference.parameters"),
            ] {
                let age = snapshot.fetched_at(endpoint).map_or_else(
                    || catalogue.text("reference.never_loaded").to_owned(),
                    |at| match state.references().age_seconds(at) {
                        Some(age) => format!(
                            "{} {age} {}",
                            catalogue.text("reference.age"),
                            catalogue.text("reference.seconds")
                        ),
                        None => catalogue.text("reference.age_unknown").to_owned(),
                    },
                );
                ui.label(format!("{}: {age}", catalogue.text(key)));
            }
        });
    }
    if let Some(notification) = state
        .reference_snapshot()
        .and_then(|snapshot| snapshot.parameters())
        .and_then(|parameters| parameters.staff_notification())
    {
        ui.label(catalogue.text("reference.staff_notification"));
        ui.label(notification);
    }
}

pub(crate) fn environment_name(environment: UexEnvironment, catalogue: &Catalogue) -> &str {
    catalogue.text(match environment {
        UexEnvironment::Live => "reference.uex_live",
        UexEnvironment::Ptu => "reference.uex_ptu",
    })
}

fn terminals(ui: &mut egui::Ui, state: &mut ShellState, catalogue: &Catalogue) {
    ui.heading(catalogue.text("reference.terminals"));
    let label = ui.label(catalogue.text("reference.terminal_search"));
    ui.add(
        egui::TextEdit::singleline(state.references_mut().terminal_query())
            .id_salt("reference-terminal-search"),
    )
    .labelled_by(label.id);
    let items: Vec<_> = state
        .reference_snapshot()
        .map(|snapshot| {
            snapshot
                .terminals(state.references().terminal_search())
                .into_iter()
                .map(|item| {
                    (
                        item.id(),
                        format!(
                            "{} · {} · {} {}",
                            item.text().name,
                            item.text().location,
                            catalogue.text("reference.id"),
                            item.id().0
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    if items.is_empty() {
        ui.label(catalogue.text("reference.no_terminals"));
    }
    egui::ScrollArea::vertical()
        .id_salt("reference-terminals")
        .max_height(ui.available_height() / 2.0)
        .show(ui, |ui| {
            for (id, label) in items {
                if ui
                    .selectable_label(state.references().selected_terminal() == Some(id), label)
                    .clicked()
                {
                    state.open_reference_terminal(id);
                }
            }
        });
}

fn commodities(ui: &mut egui::Ui, state: &mut ShellState, catalogue: &Catalogue) {
    ui.heading(catalogue.text("reference.commodities"));
    let label = ui.label(catalogue.text("reference.commodity_search"));
    ui.add(
        egui::TextEdit::singleline(state.references_mut().commodity_query())
            .id_salt("reference-commodity-search"),
    )
    .labelled_by(label.id);
    let items: Vec<_> = state
        .reference_snapshot()
        .map(|snapshot| {
            snapshot
                .commodities(state.references().commodity_search())
                .into_iter()
                .map(|item| {
                    (
                        item.id(),
                        format!(
                            "{} · {} · {} {}",
                            item.name(),
                            item.code(),
                            catalogue.text("reference.id"),
                            item.id().0
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    if items.is_empty() {
        ui.label(catalogue.text("reference.no_commodities"));
    }
    egui::ScrollArea::vertical()
        .id_salt("reference-commodities")
        .show(ui, |ui| {
            for (id, label) in items {
                if ui
                    .selectable_label(state.references().selected_commodity() == Some(id), label)
                    .clicked()
                {
                    state.select_reference_commodity(id);
                }
            }
        });
}

pub(crate) fn show_mapping(
    ui: &mut egui::Ui,
    state: &mut ShellState,
    catalogue: &Catalogue,
) -> Vec<crate::api::model::SettingsCommand> {
    use crate::api::model::UserAction;
    let mut commands = Vec::new();
    ui.separator();
    ui.heading(catalogue.text("reference.mapping"));
    ui.label(catalogue.text("reference.mapping_explanation"));
    for (channel, key) in CHANNELS {
        ui.horizontal(|ui| {
            ui.label(catalogue.text(key));
            let selected = state
                .game_witness()
                .settings
                .environment_mapping
                .environment(channel);
            for environment in [UexEnvironment::Live, UexEnvironment::Ptu] {
                if ui
                    .add_enabled(
                        state.ready() && !state.busy(),
                        egui::RadioButton::new(
                            selected == environment,
                            environment_name(environment, catalogue),
                        ),
                    )
                    .clicked()
                {
                    commands.extend(
                        state.act(UserAction::ChangeEnvironmentMapping(channel, environment)),
                    );
                }
            }
        });
    }
    commands
}
