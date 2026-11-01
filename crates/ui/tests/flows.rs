// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Critical shell flows through the drawn UI.

use egui_kittest::{Harness, kittest::Queryable};
use tallyline_ui::api::model::{BackgroundEvent, SettingsCommand};
use tallyline_ui::api::{Catalogue, FanKitUnit, ShellState, configure_fonts, show_shell};
use tallyline_workflows::api::model::AppSettings;

mod resources;

type Outcome = Result<(), String>;

struct Fixture {
    shell: ShellState,
    catalogue: Catalogue,
    unit: Option<FanKitUnit>,
    commands: Vec<SettingsCommand>,
    setup_error: Option<String>,
}

fn harness() -> Result<Harness<'static, Fixture>, String> {
    let catalogue = Catalogue::english().map_err(|error| error.to_string())?;
    let mut fixture = Fixture {
        shell: ShellState::default(),
        catalogue,
        unit: None,
        commands: Vec::new(),
        setup_error: None,
    };
    fixture
        .shell
        .receive(BackgroundEvent::SettingsLoaded(AppSettings::default()));
    let mut fonts = Some(resources::font_files()?);
    Ok(Harness::builder()
        .with_size(egui::vec2(1000.0, 900.0))
        .build_ui_state(
            move |ui, fixture: &mut Fixture| {
                if fixture.unit.is_none() {
                    if let Some(fonts) = fonts.take() {
                        configure_fonts(ui.ctx(), fonts);
                    }
                    match resources::load_unit(
                        ui.ctx(),
                        &fixture.catalogue,
                        tallyline_egui::Theme::Light,
                    ) {
                        Ok(unit) => fixture.unit = Some(unit),
                        Err(error) => {
                            fixture.setup_error = Some(error.to_string());
                            return;
                        }
                    }
                }
                if let Some(unit) = &fixture.unit {
                    fixture.commands.extend(show_shell(
                        ui,
                        &mut fixture.shell,
                        &fixture.catalogue,
                        unit,
                    ));
                }
            },
            fixture,
        ))
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_19_q_app__onboarding_issues_a_save_command_and_waits_for_persistence() -> Outcome {
    let mut harness = harness()?;
    harness.run();
    assert_eq!(harness.state().setup_error, None);
    harness.get_by_label("Continue").click();
    harness.run();
    assert_eq!(
        harness.state().commands,
        vec![SettingsCommand::CompleteOnboarding]
    );
    assert!(harness.state().shell.busy());
    assert!(!harness.state().shell.settings().onboarding_complete());
    Ok(())
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_16__background_failure_while_unfocused_and_minimised_creates_no_viewport_commands()
-> Outcome {
    let mut harness = harness()?;
    harness.run();
    harness.input_mut().focused = false;
    if let Some(viewport) = harness
        .input_mut()
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
    {
        viewport.minimized = Some(true);
        viewport.focused = Some(false);
    }
    harness.state_mut().shell.receive(BackgroundEvent::Failure(
        "Background storage failure".to_owned(),
    ));
    harness.step();
    assert_eq!(
        harness.state().shell.notice(),
        Some("Background storage failure")
    );
    assert!(!harness.state().shell.dialogs().about_open());
    assert!(
        harness
            .output()
            .viewport_output
            .values()
            .all(|output| output.commands.is_empty())
    );
    assert_eq!(harness.output().viewport_output.len(), 1);
    assert!(!harness.input().focused);
    assert_eq!(
        harness
            .input()
            .viewports
            .get(&egui::ViewportId::ROOT)
            .and_then(|viewport| viewport.minimized),
        Some(true)
    );
    Ok(())
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_3__idle_shell_requests_no_continuous_repaints() -> Outcome {
    let mut harness = harness()?;
    harness.run();
    assert_eq!(harness.run(), 1);
    assert!(
        harness
            .output()
            .viewport_output
            .values()
            .all(|viewport| viewport.repaint_delay > std::time::Duration::ZERO)
    );
    Ok(())
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_8_r_ui_19_q_app__done_leaves_settings_and_preserves_unfinished_onboarding() -> Outcome {
    use tallyline_workflows::api::model::Theme;
    for complete in [false, true] {
        let mut harness = harness()?;
        harness
            .state_mut()
            .shell
            .receive(BackgroundEvent::SettingsLoaded(
                AppSettings::from_persisted(Theme::System, complete),
            ));
        harness.state_mut().shell.show_settings();
        harness.run();
        harness.get_by_label("Done").click();
        harness.run();
        let destination = if complete {
            "Workspace"
        } else {
            "Welcome to Tallyline"
        };
        assert!(harness.query_by_label(destination).is_some());
        assert_eq!(
            harness.state().shell.settings().onboarding_complete(),
            complete
        );
    }
    Ok(())
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_18__about_identifies_the_fixed_development_version_and_platform() -> Outcome {
    let mut harness = harness()?;
    harness.run();
    harness.get_by_label("Help").click();
    harness.run();
    harness.get_by_label("About").click();
    harness.run();
    let expected = format!(
        "Development build {} · {}/{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    assert!(
        harness.query_by_label(&expected).is_some(),
        "About must identify {expected}"
    );
    Ok(())
}
