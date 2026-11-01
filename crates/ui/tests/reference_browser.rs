// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reference browsing through public shell state and commands.

use tallyline_game::api::model::GameEnvironment;
use tallyline_reference_data::api::model::{
    CachedReference, Commodity, EndpointData, UexEnvironment,
};
use tallyline_reference_data::api::{ReferenceApplication, ReferenceDataCache};
use tallyline_shared_kernel::api::Clock;
use tallyline_shared_kernel::api::model::{CommodityId, Timestamp};
use tallyline_ui::api::ShellState;
use tallyline_ui::api::model::{BackgroundEvent, View};

#[expect(
    dead_code,
    reason = "The shared UI fixture also supplies artwork for other harnesses."
)]
mod resources;

struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_seconds(120)
    }
}
struct MemoryCache(Vec<CachedReference>);
impl ReferenceDataCache for MemoryCache {
    type Error = &'static str;
    fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
        Ok(self.0.clone())
    }
    fn replace(&mut self, _: &CachedReference) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_ui_16__passive_refresh_preserves_environment_search_selection_and_settings_view() {
    let app = ReferenceApplication::load(
        MemoryCache(vec![CachedReference::new(
            Timestamp::from_seconds(60),
            EndpointData::Commodities(vec![Commodity::new(
                CommodityId(42),
                "Agricium",
                "AGRI",
                true,
                None,
            )]),
        )]),
        FixedClock,
    )
    .unwrap();
    let mut shell = ShellState::default();
    shell.receive(BackgroundEvent::ReferenceUpdated {
        live: app.snapshot(UexEnvironment::Live),
        ptu: app.snapshot(UexEnvironment::Ptu),
        observed_at: Timestamp::from_seconds(120),
    });
    shell.references_mut().choose_channel(GameEnvironment::Eptu);
    *shell.references_mut().commodity_query() = "AGRI".to_owned();
    assert!(shell.select_reference_commodity(CommodityId(42)));
    shell.show_settings();
    shell.receive(BackgroundEvent::ReferenceRefreshStarted);
    shell.receive(BackgroundEvent::ReferenceUpdated {
        live: app.snapshot(UexEnvironment::Live),
        ptu: app.snapshot(UexEnvironment::Ptu),
        observed_at: Timestamp::from_seconds(180),
    });
    assert_eq!(shell.view(), View::Settings);
    assert_eq!(shell.references().channel(), GameEnvironment::Eptu);
    assert_eq!(shell.references().commodity_search(), "AGRI");
    assert_eq!(
        shell.references().selected_commodity(),
        Some(CommodityId(42))
    );
    assert_eq!(
        shell.reference_snapshot().unwrap().environment(),
        UexEnvironment::Ptu
    );
    assert!(!shell.dialogs().about_open());
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_2_r_api_6_r_cap_3a__drawn_browser_selects_ptu_only_records_and_dispatches_terminal_prices()
{
    use egui_kittest::{Harness, kittest::Queryable};
    use tallyline_reference_data::api::model::{Terminal, TerminalText};
    use tallyline_shared_kernel::api::model::TerminalId;
    use tallyline_ui::api::model::ReferenceCommand;
    use tallyline_ui::api::{Catalogue, show_reference_browser};
    let app = ReferenceApplication::load(
        MemoryCache(vec![CachedReference::new(
            Timestamp::from_seconds(60),
            EndpointData::Terminals(vec![Terminal::new(
                TerminalId(12),
                TerminalText {
                    name: "Orbital Depot".to_owned(),
                    nickname: "Depot".to_owned(),
                    display_name: "Orbital Depot".to_owned(),
                    location: "Pyro".to_owned(),
                },
                false,
            )]),
        )]),
        FixedClock,
    )
    .unwrap();
    let mut shell = ShellState::default();
    shell.receive(BackgroundEvent::ReferenceUpdated {
        live: app.snapshot(UexEnvironment::Live),
        ptu: app.snapshot(UexEnvironment::Ptu),
        observed_at: Timestamp::from_seconds(120),
    });
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(resources::font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 900.0))
        .build_ui_state(
            move |ui, state| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_reference_browser(ui, state, &catalogue);
            },
            shell,
        );
    harness.run();
    assert!(
        harness
            .query_by_label("Orbital Depot · Pyro · UEX ID 12")
            .is_none()
    );
    harness.get_by_label("EPTU").click();
    harness.run();
    harness
        .get_by_label("Orbital Depot · Pyro · UEX ID 12")
        .click();
    harness.run();
    assert_eq!(
        harness.state_mut().references_mut().take_command(),
        Some(ReferenceCommand::OpenTerminal(TerminalId(12)))
    );
    assert_eq!(
        harness.state().references().selected_terminal(),
        Some(TerminalId(12))
    );
    assert_eq!(
        harness.state().game_witness().settings.witness,
        tallyline_game::api::model::WitnessMode::Off
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_cap_3a_r_nf_5__mapping_changes_wait_for_durable_confirmation_without_enabling_witness() {
    use tallyline_ui::api::model::{GameWitnessView, SettingsCommand, UserAction};
    use tallyline_workflows::api::model::AppSettings;
    let mut shell = ShellState::default();
    shell.receive(BackgroundEvent::SettingsLoaded(AppSettings::default()));
    assert_eq!(
        shell.act(UserAction::ChangeEnvironmentMapping(
            GameEnvironment::Live,
            UexEnvironment::Ptu
        )),
        Some(SettingsCommand::ChangeEnvironmentMapping(
            GameEnvironment::Live,
            UexEnvironment::Ptu
        ))
    );
    assert_eq!(
        shell
            .game_witness()
            .settings
            .environment_mapping
            .environment(GameEnvironment::Live),
        UexEnvironment::Live
    );
    shell.receive(BackgroundEvent::Failure("Save failed".to_owned()));
    assert_eq!(
        shell
            .game_witness()
            .settings
            .environment_mapping
            .environment(GameEnvironment::Live),
        UexEnvironment::Live
    );
    assert!(
        shell
            .act(UserAction::ChangeEnvironmentMapping(
                GameEnvironment::Live,
                UexEnvironment::Ptu
            ))
            .is_some()
    );
    let mut durable = GameWitnessView::default();
    durable.settings.environment_mapping = durable
        .settings
        .environment_mapping
        .with_environment(GameEnvironment::Live, UexEnvironment::Ptu);
    shell.receive(BackgroundEvent::GameWitnessLoaded(durable));
    assert_eq!(
        shell
            .game_witness()
            .settings
            .environment_mapping
            .environment(GameEnvironment::Live),
        UexEnvironment::Ptu
    );
    assert_eq!(
        shell.game_witness().settings.witness,
        tallyline_game::api::model::WitnessMode::Off
    );
    assert!(!shell.busy());
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_16_r_ui_17__background_refresh_keeps_the_search_field_focused_and_accepting_text() {
    use egui_kittest::{Harness, kittest::Queryable};
    use tallyline_ui::api::{Catalogue, show_reference_browser};
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(resources::font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 900.0))
        .build_ui_state(
            move |ui, state| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_reference_browser(ui, state, &catalogue);
            },
            ShellState::default(),
        );
    harness.run();
    harness.get_by_label("Search terminals").focus();
    harness.run();
    harness.get_by_label("Search terminals").type_text("Area");
    harness.run();
    assert_eq!(harness.state().references().terminal_search(), "Area");
    harness.key_press(egui::Key::End);
    harness.run();
    harness
        .state_mut()
        .receive(BackgroundEvent::ReferenceRefreshStarted);
    harness.run();
    assert!(harness.get_by_label("Search terminals").is_focused());
    harness.get_by_label("Search terminals").type_text("18");
    harness.run();
    assert_eq!(harness.state().references().terminal_search(), "Area18");
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_ui_8_r_ui_13__empty_cache_and_offline_refresh_report_real_data_age() {
    use egui_kittest::{Harness, kittest::Queryable};
    use tallyline_reference_data::api::model::Endpoint;
    use tallyline_ui::api::model::ReferenceCommand;
    use tallyline_ui::api::{Catalogue, show_reference_browser};
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(resources::font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 900.0))
        .build_ui_state(
            move |ui, state| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_reference_browser(ui, state, &catalogue);
            },
            ShellState::default(),
        );
    harness.run();
    assert!(harness.query_by_label("No reference data yet").is_some());
    let mut app = ReferenceApplication::load(
        MemoryCache(vec![CachedReference::new(
            Timestamp::from_seconds(60),
            EndpointData::Commodities(vec![Commodity::new(
                CommodityId(42),
                "Agricium",
                "AGRI",
                true,
                None,
            )]),
        )]),
        FixedClock,
    )
    .unwrap();
    app.refresh_failed(Endpoint::Commodities);
    harness
        .state_mut()
        .receive(BackgroundEvent::ReferenceUpdated {
            live: app.snapshot(UexEnvironment::Live),
            ptu: app.snapshot(UexEnvironment::Ptu),
            observed_at: Timestamp::from_seconds(120),
        });
    harness.run();
    assert!(
        harness
            .query_by_label("Offline - cached reference data remains available")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("Commodities: Age at last update: 60 seconds")
            .is_some()
    );
    harness.get_by_label("Refresh reference data").click();
    harness.run();
    assert_eq!(
        harness.state_mut().references_mut().take_command(),
        Some(ReferenceCommand::Refresh)
    );
    assert!(
        harness
            .query_by_label("Agricium · AGRI · UEX ID 42")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("Refreshing reference data in the background")
            .is_some()
    );
    harness
        .state_mut()
        .receive(BackgroundEvent::ReferenceUpdated {
            live: app.snapshot(UexEnvironment::Live),
            ptu: app.snapshot(UexEnvironment::Ptu),
            observed_at: Timestamp::from_seconds(30),
        });
    harness.run();
    assert!(
        harness
            .query_by_label("Commodities: Age unknown - clock mismatch")
            .is_some()
    );
    assert!(!harness.state().references().refreshing());
}
