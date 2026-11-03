// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Minimum-viewport and magnified drawn page contracts.

use egui_kittest::kittest::Queryable;
use tallyline_reporting::api::model::{Report, ReportId, ReportKey};
use tallyline_shared_kernel::api::model::{CommodityId, TerminalId, Timestamp, TradeSide};
use tallyline_ui::api::model::{BackgroundEvent, View};
use tallyline_workflows::api::model::{AppSettings, Theme};

type ShowView = fn(&mut tallyline_ui::api::ShellState);

#[test]
fn minimum_viewport_keeps_existing_pages_and_manual_inputs_reachable_in_every_theme_at_double_scale()
 {
    for theme in [Theme::Dark, Theme::Light, Theme::HighContrast] {
        for zoom in [1.0, 2.0] {
            for label in [
                "Price per SCU for commodity 42",
                "SCU for commodity 42",
                "Status code for commodity 42",
            ] {
                let mut harness =
                    super::configured_shell(egui::vec2(800.0, 640.0), theme, zoom).unwrap();
                harness
                    .state_mut()
                    .shell
                    .receive(BackgroundEvent::SettingsLoaded(
                        AppSettings::from_persisted(theme, true),
                    ));
                harness
                    .state_mut()
                    .shell
                    .receive(super::age::reference_event().unwrap());
                let mut report = Report::manual(
                    ReportId(1),
                    ReportKey::new(
                        TerminalId(11),
                        TradeSide::Buy,
                        tallyline_game::api::model::GameEnvironment::Live,
                    )
                    .unwrap(),
                    Timestamp::from_seconds(100),
                    Some("4.10.2"),
                )
                .unwrap();
                report.add_commodity(CommodityId(42)).unwrap();
                harness
                    .state_mut()
                    .shell
                    .receive(BackgroundEvent::ReportsLoaded {
                        reports: vec![report],
                        selected: Some(ReportId(1)),
                    });
                let viewport = egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0 / zoom, 640.0 / zoom),
                );
                let views: [(View, ShowView); 4] = [
                    (View::Home, tallyline_ui::api::ShellState::leave_settings),
                    (
                        View::Reference,
                        tallyline_ui::api::ShellState::show_reference,
                    ),
                    (View::Settings, tallyline_ui::api::ShellState::show_settings),
                    (View::Manual, tallyline_ui::api::ShellState::show_manual),
                ];
                for (view, show) in views {
                    show(&mut harness.state_mut().shell);
                    harness.run();
                    let status = harness.get_by_label("Saved submissions: 0").rect();
                    assert!(
                        viewport.contains_rect(status),
                        "{theme:?} {zoom} {view:?}: {status:?}"
                    );
                    let settings = harness
                        .get_by_role_and_label(egui::Role::Button, "Settings")
                        .rect();
                    assert!(settings.width() >= 24.0 && settings.height() >= 24.0);
                    assert!(
                        viewport.contains_rect(settings),
                        "Navigation remains visible: {theme:?} {zoom}: {settings:?}"
                    );
                    if view == View::Manual {
                        harness
                            .get_by_role_and_label(egui::Role::TextInput, label)
                            .focus();
                        harness.run();
                        let input = harness.get_by_role_and_label(egui::Role::TextInput, label);
                        let rect = input.rect();
                        assert!(
                            input.is_focused(),
                            "{theme:?} {zoom}: {label} stays focused"
                        );
                        assert!(
                            viewport.contains_rect(rect),
                            "{theme:?} {zoom}: {label}: {rect:?}"
                        );
                        assert!(
                            rect.bottom() <= status.top(),
                            "Focused input stays above status: {rect:?} {status:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn settings_stock_inputs_remain_visible_and_save_explicitly_at_double_scale() {
    for theme in [Theme::Dark, Theme::Light, Theme::HighContrast] {
        let mut harness = super::configured_shell(egui::vec2(800.0, 640.0), theme, 2.0).unwrap();
        harness.state_mut().shell.show_settings();
        harness.run();
        harness.get_by_label("SCU review tolerance").focus();
        harness.run();
        harness.key_press(egui::Key::Space);
        harness.run();
        let label = "Absolute SCU minimum (0 to 16)";
        harness
            .get_by_role_and_label(egui::Role::TextInput, label)
            .focus();
        harness.run();
        let input = harness.get_by_role_and_label(egui::Role::TextInput, label);
        let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 320.0));
        assert!(input.is_focused());
        assert!(
            viewport.contains_rect(input.rect()),
            "{theme:?} Settings input: {:?}",
            input.rect()
        );
        assert!(input.rect().bottom() <= harness.get_by_label("Saved submissions: 0").rect().top());
        harness.get_by_label("Save SCU tolerance").focus();
        harness.run();
        harness.key_press(egui::Key::Space);
        harness.run();
        assert!(harness.state().commands.iter().any(|command| matches!(
            command,
            tallyline_ui::api::model::SettingsCommand::ChangeStockSettings(_)
        )));
    }
}
