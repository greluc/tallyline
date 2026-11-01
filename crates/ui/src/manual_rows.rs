// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keyboard entry with aged UEX references beside empty manual inputs.

use crate::api::{Catalogue, ShellState};
use tallyline_reference_data::api::model::ReferenceSnapshot;
use tallyline_reporting::api::ManualEdit;
use tallyline_shared_kernel::api::model::TradeSide;

pub(crate) fn show(
    ui: &mut egui::Ui,
    state: &mut ShellState,
    catalogue: &Catalogue,
    snapshot: &ReferenceSnapshot,
) {
    let Some(report) = state.manual().selected().cloned() else {
        return;
    };
    ui.separator();
    ui.label(format!(
        "{} {}",
        catalogue.text("reference.version"),
        report
            .game_version()
            .unwrap_or(catalogue.text("reference.no_version"))
    ));
    ui.label(format!(
        "{} {}",
        catalogue.text("manual.observed"),
        report.observed_at().seconds()
    ));
    let priors = snapshot.terminal_priors(report.key().terminal());
    let mut add = None;
    if let Some(priors) = priors {
        for prior in priors
            .rows()
            .iter()
            .filter(|prior| prior.side() == report.key().side())
        {
            if !report
                .rows()
                .iter()
                .any(|row| row.commodity() == prior.commodity())
                && ui
                    .button(format!(
                        "{} {}",
                        catalogue.text("manual.add"),
                        name(snapshot, prior.commodity())
                    ))
                    .clicked()
            {
                add = Some(prior.commodity());
            }
        }
    }
    ui.add(
        egui::TextEdit::singleline(&mut state.manual_mut().commodity_query)
            .hint_text(catalogue.text("manual.add_search"))
            .id_salt("manual-commodity"),
    )
    .widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::Role::TextInput,
            true,
            catalogue.text("manual.add_search"),
        )
    });
    if !state.manual().commodity_query.is_empty() {
        for commodity in snapshot
            .commodities(&state.manual().commodity_query)
            .into_iter()
        {
            if !report
                .rows()
                .iter()
                .any(|row| row.commodity() == commodity.id())
                && ui
                    .button(format!(
                        "{} {} · UEX ID {}",
                        catalogue.text("manual.add"),
                        commodity.name(),
                        commodity.id().0
                    ))
                    .clicked()
            {
                add = Some(commodity.id());
            }
        }
    }
    if let Some(commodity) = add {
        crate::manual_views::queue_edit(
            state,
            ManualEdit::AddCommodity(commodity),
            snapshot.clone(),
        );
    }
    let ages: std::collections::BTreeMap<_, _> = priors
        .into_iter()
        .flat_map(|priors| priors.rows())
        .map(|prior| {
            (
                prior.commodity(),
                state
                    .references()
                    .age_seconds(prior.provenance().modified_at),
            )
        })
        .collect();
    let mut edit = None;
    let mut invalid = false;
    for row in &mut state.manual_mut().rows {
        ui.push_id(row.id.0, |ui| {
            ui.separator();
            ui.label(name(snapshot, row.commodity));
            let prior = priors.and_then(|priors| {
                priors.rows().iter().find(|prior| {
                    prior.commodity() == row.commodity && prior.side() == report.key().side()
                })
            });
            if let Some(prior) = prior {
                let values = prior.values();
                let age = ages.get(&row.commodity).copied().flatten().map_or_else(
                    || catalogue.text("reference.age_unknown").to_owned(),
                    |age| {
                        format!(
                            "{} {age} {}",
                            catalogue.text("reference.age"),
                            catalogue.text("reference.seconds")
                        )
                    },
                );
                ui.label(format!(
                    "{}: {} / {} SCU / {} · {}",
                    catalogue.text("manual.reference"),
                    values.price,
                    values.stock,
                    values
                        .status
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "?".to_owned()),
                    age
                ));
                if ui.button(catalogue.text("manual.takeover")).clicked() {
                    edit = Some(ManualEdit::TakeOver(row.id));
                }
            } else {
                ui.label(catalogue.text("manual.no_reference"));
            }
            ui.horizontal(|ui| {
                if field(
                    ui,
                    &mut row.price,
                    format!("{} {}", catalogue.text("manual.price_for"), row.commodity.0),
                ) {
                    match optional(&row.price) {
                        Ok(value) => edit = Some(ManualEdit::Price(row.id, value)),
                        Err(()) => invalid = true,
                    }
                }
                if field(
                    ui,
                    &mut row.stock,
                    format!("{} {}", catalogue.text("manual.scu_for"), row.commodity.0),
                ) {
                    match optional(&row.stock) {
                        Ok(value) => edit = Some(ManualEdit::Stock(row.id, value)),
                        Err(()) => invalid = true,
                    }
                }
                if field(
                    ui,
                    &mut row.status,
                    format!(
                        "{} {}",
                        catalogue.text("manual.status_for"),
                        row.commodity.0
                    ),
                ) {
                    match optional(&row.status) {
                        Ok(value) => edit = Some(ManualEdit::Status(row.id, value)),
                        Err(()) => invalid = true,
                    }
                }
            });
        });
    }
    if invalid {
        state.manual_mut().notice = Some(catalogue.text("manual.invalid").to_owned());
    }
    if let Some(edit) = edit {
        crate::manual_views::queue_edit(state, edit, snapshot.clone());
    }
    if let Some(levels) = snapshot.status_levels() {
        let levels = match report.key().side() {
            TradeSide::Buy => &levels.buy,
            TradeSide::Sell => &levels.sell,
        };
        ui.label(
            levels
                .iter()
                .map(|level| format!("{}: {}", level.code, level.name))
                .collect::<Vec<_>>()
                .join(" · "),
        );
    }
}
fn name(
    snapshot: &ReferenceSnapshot,
    commodity: tallyline_shared_kernel::api::model::CommodityId,
) -> String {
    snapshot
        .commodities("")
        .iter()
        .find(|item| item.id() == commodity)
        .map_or_else(
            || format!("UEX ID {}", commodity.0),
            |item| format!("{} · UEX ID {}", item.name(), commodity.0),
        )
}
fn field(ui: &mut egui::Ui, input: &mut String, label: String) -> bool {
    ui.vertical(|ui| {
        ui.label(&label);
        let response = ui.add(
            egui::TextEdit::singleline(input)
                .desired_width(130.0)
                .id_salt(&label),
        );
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, &label));
        response.lost_focus()
    })
    .inner
}
fn optional<T: std::str::FromStr>(text: &str) -> Result<Option<T>, ()> {
    if text.trim().is_empty() {
        Ok(None)
    } else {
        text.trim().parse().map(Some).map_err(|_| ())
    }
}
