// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keyboard-driven manual Report entry contracts.

use egui_kittest::{Harness, kittest::Queryable};
use tallyline_game::api::model::GameEnvironment;
use tallyline_reporting::api::model::{Report, ReportId, ReportKey};
use tallyline_shared_kernel::api::model::{CommodityId, TerminalId, Timestamp, TradeSide};
use tallyline_ui::api::model::{BackgroundEvent, ManualCommand};
use tallyline_ui::api::{Catalogue, ShellState, show_manual_editor};

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_3_r_ui_2__resumed_draft_keeps_empty_inputs_until_keyboard_entry_and_queues_an_explicit_user_edit()
 {
    let mut report = Report::manual(
        ReportId(1),
        ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
        Timestamp::from_seconds(100),
        Some("4.10.2"),
    )
    .unwrap();
    report.add_commodity(CommodityId(42)).unwrap();
    let mut shell = ShellState::default();
    shell.receive(BackgroundEvent::ReportsLoaded {
        reports: vec![report],
        selected: Some(ReportId(1)),
    });
    struct Cache;
    impl tallyline_reference_data::api::ReferenceDataCache for Cache {
        type Error = std::convert::Infallible;
        fn load(
            &mut self,
        ) -> Result<Vec<tallyline_reference_data::api::model::CachedReference>, Self::Error>
        {
            use tallyline_reference_data::api::model::{
                CachedReference, Commodity, CommodityAverages, EndpointData,
            };
            Ok(vec![CachedReference::new(
                Timestamp::from_seconds(100),
                EndpointData::Commodities(vec![
                    Commodity::new(
                        CommodityId(43),
                        "Gold",
                        "G1",
                        true,
                        CommodityAverages::new(None, None),
                    ),
                    Commodity::new(
                        CommodityId(44),
                        "Gold",
                        "G2",
                        true,
                        CommodityAverages::new(None, None),
                    ),
                ]),
            )])
        }
        fn replace(
            &mut self,
            _: &tallyline_reference_data::api::model::CachedReference,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    struct Fixed;
    impl tallyline_shared_kernel::api::Clock for Fixed {
        fn now(&self) -> Timestamp {
            Timestamp::from_seconds(120)
        }
    }
    let reference =
        tallyline_reference_data::api::ReferenceApplication::load(Cache, Fixed).unwrap();
    shell.receive(BackgroundEvent::ReferenceUpdated {
        live: reference.snapshot(tallyline_reference_data::api::model::UexEnvironment::Live),
        ptu: reference.snapshot(tallyline_reference_data::api::model::UexEnvironment::Ptu),
        observed_at: Timestamp::from_seconds(120),
    });
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1200.0, 900.0))
        .build_ui_state(
            move |ui, state| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_manual_editor(ui, state, &catalogue);
            },
            shell,
        );
    harness.run();
    harness
        .get_by_label("Add commodity from vocabulary")
        .focus();
    harness.run();
    harness
        .get_by_label("Add commodity from vocabulary")
        .type_text("Gold");
    harness.run();
    assert!(
        harness
            .query_by_label("Add commodity Gold · UEX ID 43")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("Add commodity Gold · UEX ID 44")
            .is_some()
    );
    assert!(harness.state_mut().manual_mut().take_command().is_none());
    harness
        .get_by_role_and_label(egui::Role::TextInput, "Price per SCU for commodity 42")
        .focus();
    harness.run();
    harness
        .get_by_role_and_label(egui::Role::TextInput, "Price per SCU for commodity 42")
        .type_text("123.45");
    harness.run();
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert!(matches!(
        harness.state_mut().manual_mut().take_command(),
        Some(ManualCommand::Edit {
            edit: tallyline_reporting::api::ManualEdit::Price(_, Some(_)),
            ..
        })
    ));
}

fn font_files() -> Result<tallyline_ui::api::model::FontFiles, &'static str> {
    Ok(tallyline_ui::api::model::FontFiles {
        material_symbols: include_bytes!(
            "../assets/fonts/material-symbols/MaterialSymbolsRounded-Tallyline.ttf"
        )
        .to_vec()
        .try_into()?,
        plex_regular: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-Regular.ttf").to_vec(),
        plex_semibold: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-SemiBold.ttf").to_vec(),
        plex_bold: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-Bold.ttf").to_vec(),
        mono_regular: include_bytes!("../assets/fonts/jetbrains-mono/JetBrainsMonoNL-Regular.ttf")
            .to_vec(),
        mono_bold: include_bytes!("../assets/fonts/jetbrains-mono/JetBrainsMonoNL-Bold.ttf")
            .to_vec(),
    })
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_ui_16__repository_failures_render_actionable_passive_notices_without_losing_the_draft()
{
    use tallyline_reporting::api::RepositoryError;
    let mut report = Report::manual(
        ReportId(1),
        ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
        Timestamp::from_seconds(100),
        None,
    )
    .unwrap();
    let row = report.add_commodity(CommodityId(42)).unwrap();
    report
        .set_price(row, Some("123.45".parse().unwrap()))
        .unwrap();
    let mut shell = ShellState::default();
    shell.receive(BackgroundEvent::ReportsLoaded {
        reports: vec![report.clone()],
        selected: Some(report.id()),
    });
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1200.0, 900.0))
        .build_ui_state(
            move |ui, state| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_manual_editor(ui, state, &catalogue);
            },
            shell,
        );
    for (error, notice) in [
        (
            RepositoryError::Conflict,
            "Draft changed; reopen it before editing",
        ),
        (
            RepositoryError::Invalid,
            "Draft value, context or stored state is invalid",
        ),
        (RepositoryError::Unavailable, "Draft storage is unavailable"),
    ] {
        harness
            .state_mut()
            .receive(BackgroundEvent::ReportFailed(error));
        harness.run();
        assert!(harness.query_by_label(notice).is_some());
        assert_eq!(harness.state().manual().selected(), Some(&report));
        assert!(harness.state_mut().manual_mut().take_command().is_none());
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_1__all_ambiguous_terminal_results_remain_selectable_beyond_twelve_matches() {
    use tallyline_reference_data::api::model::{
        CachedReference, EndpointData, Terminal, TerminalText, UexEnvironment,
    };
    use tallyline_reference_data::api::{ReferenceApplication, ReferenceDataCache};
    struct Cache;
    impl ReferenceDataCache for Cache {
        type Error = std::convert::Infallible;
        fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
            Ok(vec![CachedReference::new(
                Timestamp::from_seconds(100),
                EndpointData::Terminals(
                    (1..=13)
                        .map(|id| {
                            Terminal::new(
                                TerminalId(id),
                                TerminalText {
                                    name: format!("Depot {id}"),
                                    nickname: "DP".to_owned(),
                                    display_name: "Shared shop".to_owned(),
                                    location: format!("System {id}"),
                                },
                                true,
                            )
                        })
                        .collect(),
                ),
            )])
        }
        fn replace(&mut self, _: &CachedReference) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    struct Fixed;
    impl tallyline_shared_kernel::api::Clock for Fixed {
        fn now(&self) -> Timestamp {
            Timestamp::from_seconds(120)
        }
    }
    let reference = ReferenceApplication::load(Cache, Fixed).unwrap();
    let mut shell = ShellState::default();
    shell.receive(BackgroundEvent::ReportsLoaded {
        reports: vec![],
        selected: None,
    });
    shell.receive(BackgroundEvent::ReferenceUpdated {
        live: reference.snapshot(UexEnvironment::Live),
        ptu: reference.snapshot(UexEnvironment::Ptu),
        observed_at: Timestamp::from_seconds(120),
    });
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1200.0, 900.0))
        .build_ui_state(
            move |ui, state| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_manual_editor(ui, state, &catalogue);
            },
            shell,
        );
    harness.run();
    harness.get_by_label("LIVE").click();
    harness.run();
    assert!(harness.query_by_label("Search terminals").is_some());
    assert!(harness.state_mut().manual_mut().take_command().is_none());
    assert!(
        harness
            .query_by_label("Depot 13 · System 13 · UEX ID 13")
            .is_some()
    );
}
