// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Explicit numeric review navigation and command binding.

use tallyline_ui::api::model::{ManualReviewState, ReviewDirection};

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_12_q_manual__empty_review_never_selects_or_confirms_a_field() {
    let mut state = ManualReviewState::default();
    assert!(state.review().is_none());
    assert!(state.focused().is_none());
    assert!(state.navigate(ReviewDirection::Next).is_none());
    assert!(state.navigate(ReviewDirection::Previous).is_none());
    assert!(state.confirm_focused().is_none());
}

#[path = "manual_review/support.rs"]
mod support;

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_12_q_manual__explicit_navigation_wraps_required_fields_and_commands_bind_the_displayed_version()
 {
    use tallyline_reporting::api::model::ReviewField;
    use tallyline_ui::api::model::ManualReviewAction;
    let (review, snapshot) = support::review().unwrap();
    let id = review.report().id();
    let version = review.report().version();
    let row = review.report().rows()[0].id();
    let mut state = ManualReviewState::default();
    state.received(review.clone(), snapshot.clone());
    assert!(state.focused().is_none());
    let first = state.navigate(ReviewDirection::Next).unwrap();
    assert_eq!(first.id, id);
    assert_eq!(first.expected, version);
    assert_eq!(first.snapshot, snapshot);
    assert_eq!(
        first.action,
        ManualReviewAction::Focus(ReviewField::Price(row))
    );
    assert_eq!(
        state.confirm_focused().unwrap().action,
        ManualReviewAction::Confirm(ReviewField::Price(row))
    );
    assert_eq!(
        state.navigate(ReviewDirection::Next).unwrap().action,
        ManualReviewAction::Focus(ReviewField::Status(row))
    );
    assert_eq!(
        state.navigate(ReviewDirection::Next).unwrap().action,
        ManualReviewAction::Focus(ReviewField::Price(row))
    );
    assert_eq!(
        state.navigate(ReviewDirection::Previous).unwrap().action,
        ManualReviewAction::Focus(ReviewField::Status(row))
    );
    state.received(review.clone(), snapshot);
    assert_eq!(state.focused(), Some(ReviewField::Status(row)));
    assert_eq!(state.review(), Some(&review));
    assert!(
        state.review().unwrap().report().rows()[0]
            .price_confirmation()
            .is_none()
    );
    state.invalidate();
    assert!(state.review().is_none());
    assert!(state.confirm_focused().is_none());
    assert!(state.navigate(ReviewDirection::Next).is_none());
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_12_q_manual__direct_keyboard_focus_records_review_without_approving_or_accepting_unknown_fields()
 {
    use tallyline_reporting::api::model::ReviewField;
    use tallyline_shared_kernel::api::model::RowId;
    use tallyline_ui::api::model::ManualReviewAction;
    let (review, snapshot) = support::review().unwrap();
    let row = review.report().rows()[0].id();
    let mut state = ManualReviewState::default();
    state.received(review, snapshot);
    assert!(!state.needs_confirmation(ReviewField::Price(RowId(99))));
    assert!(state.needs_confirmation(ReviewField::Price(row)));
    assert!(state.focus(ReviewField::Price(RowId(99))).is_none());
    assert!(state.focused().is_none());
    let focused = state.focus(ReviewField::Price(row)).unwrap();
    assert_eq!(
        focused.action,
        ManualReviewAction::Focus(ReviewField::Price(row))
    );
    assert_eq!(state.review().unwrap().summary().needs_confirmation, 2);
    assert!(
        state.review().unwrap().report().rows()[0]
            .price_confirmation()
            .is_none()
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_11_q_manual_r_ui_12_q_manual__drawn_review_shows_exact_reference_deltas_and_keyboard_confirmation_intent()
 {
    use egui_kittest::{Harness, kittest::Queryable};
    use tallyline_reporting::api::model::ReviewField;
    use tallyline_ui::api::model::{ManualReviewAction, ManualReviewCommand};
    use tallyline_ui::api::{Catalogue, show_manual_review};
    let (review, snapshot) = support::review().unwrap();
    let row = review.report().rows()[0].id();
    let mut state = ManualReviewState::default();
    state.received(review, snapshot);
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(tallyline_ui::api::model::FontFiles {
        plex_regular: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-Regular.ttf").to_vec(),
        plex_semibold: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-SemiBold.ttf").to_vec(),
        plex_bold: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-Bold.ttf").to_vec(),
        mono_regular: include_bytes!("../assets/fonts/jetbrains-mono/JetBrainsMonoNL-Regular.ttf")
            .to_vec(),
        mono_bold: include_bytes!("../assets/fonts/jetbrains-mono/JetBrainsMonoNL-Bold.ttf")
            .to_vec(),
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 850.0))
        .build_ui_state(
            move |ui, state: &mut (ManualReviewState, Vec<ManualReviewCommand>)| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                if let Some(command) =
                    show_manual_review(ui, &mut state.0, &catalogue, tallyline_egui::Theme::Dark)
                {
                    state.1.push(command);
                }
            },
            (state, Vec::new()),
        );
    harness.run();
    assert!(harness.query_by_label("Price and status review").is_some());
    assert!(
        harness
            .query_by_label(
                "2 major · 0 within tolerance · 2 confirmations required · 2 unreviewed"
            )
            .is_some()
    );
    assert!(
        harness
            .query_by_label("Price · commodity 42: 200")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("UEX reference: 100 · age at review: 100 seconds")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("Difference: 100 per SCU · +100.00 %")
            .is_some()
    );
    assert!(harness.state().1.is_empty());
    harness.key_press(egui::Key::F8);
    harness.run();
    assert_eq!(
        harness.state().1.last().unwrap().action,
        ManualReviewAction::Focus(ReviewField::Price(row))
    );
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().1.last().unwrap().action,
        ManualReviewAction::Confirm(ReviewField::Price(row))
    );
    assert!(
        harness.state().0.review().unwrap().report().rows()[0]
            .price_confirmation()
            .is_none()
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_11_q_manual_r_ui_12_q_manual__confirmation_dialog_requires_a_separate_explicit_approval_of_the_displayed_value()
 {
    use egui_kittest::{Harness, kittest::Queryable};
    use tallyline_reporting::api::model::ReviewField;
    use tallyline_ui::api::model::ManualReviewCommand;
    use tallyline_ui::api::{Catalogue, DialogPresenter};
    let (review, snapshot) = support::review().unwrap();
    let row = review.report().rows()[0].id();
    let mut state = ManualReviewState::default();
    state.received(review, snapshot);
    let mut presenter = DialogPresenter::default();
    assert!(!presenter.manual_confirmation_open());
    presenter.open_manual_confirmation(&state, ReviewField::Price(row));
    assert!(presenter.manual_confirmation_open());
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(support::fonts());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 850.0))
        .build_ui_state(
        move |ui, state: &mut (ManualReviewState, DialogPresenter, Vec<ManualReviewCommand>)| {
            if let Some(fonts) = fonts.take() {
                tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
            }
            if let Some(command) = state.1.show_manual_confirmation(
                ui.ctx(),
                &state.0,
                &catalogue,
                tallyline_egui::Theme::Dark,
                |ui| {
                    ui.label("No screenshot attached");
                },
            ) {
                state.2.push(command);
            }
        },
        (state, presenter, Vec::new()),
    );
    harness.run();
    assert!(harness.query_by_label("Confirm this exact value").is_some());
    assert!(harness.query_by_label("200 per SCU").is_some());
    assert!(
        harness
            .query_by_label("Difference: 100 per SCU · +100.00 %")
            .is_some()
    );
    assert!(harness.state().2.is_empty());
    harness.get_by_label("Approve this exact value").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().2.len(), 1);
    assert!(!harness.state().1.manual_confirmation_open());
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_ui_12_q_manual_r_ui_16__cancel_or_invalidated_review_closes_the_dialog_without_dispatching_approval()
 {
    use egui_kittest::{Harness, kittest::Queryable};
    use tallyline_reporting::api::model::ReviewField;
    use tallyline_ui::api::model::ManualReviewCommand;
    use tallyline_ui::api::{Catalogue, DialogPresenter};
    let (review, snapshot) = support::review().unwrap();
    let field = ReviewField::Price(review.report().rows()[0].id());
    let mut state = ManualReviewState::default();
    state.received(review, snapshot);
    let mut presenter = DialogPresenter::default();
    presenter.open_manual_confirmation(&state, field);
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(support::fonts());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 850.0))
        .build_ui_state(
        move |ui, state: &mut (ManualReviewState, DialogPresenter, Vec<ManualReviewCommand>)| {
            if let Some(fonts) = fonts.take() {
                tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
            }
            if let Some(command) = state.1.show_manual_confirmation(
                ui.ctx(),
                &state.0,
                &catalogue,
                tallyline_egui::Theme::Dark,
                |ui| {
                    ui.label("No screenshot attached");
                },
            ) {
                state.2.push(command);
            }
        },
        (state, presenter, Vec::new()),
    );
    harness.run();
    harness.get_by_label("Cancel").click();
    harness.run();
    assert!(!harness.state().1.manual_confirmation_open());
    assert!(harness.state().2.is_empty());
    let (state, presenter, _) = harness.state_mut();
    presenter.open_manual_confirmation(state, field);
    state.invalidate();
    harness.run();
    assert!(!harness.state().1.manual_confirmation_open());
    assert!(harness.query_by_label("Approve this exact value").is_none());
    assert!(harness.state().2.is_empty());
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_val_2a_q_manual_r_ui_11_q_manual__missing_prior_is_informational_and_never_presented_as_precision_failure()
 {
    use egui_kittest::{Harness, kittest::Queryable};
    use tallyline_ui::api::{Catalogue, show_manual_review};
    let (review, snapshot) = support::without_prior().unwrap();
    let mut state = ManualReviewState::default();
    state.received(review, snapshot);
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(support::fonts());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 850.0))
        .build_ui_state(
            move |ui, state| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                assert!(
                    show_manual_review(ui, state, &catalogue, tallyline_egui::Theme::Dark)
                        .is_none()
                );
            },
            state,
        );
    harness.run();
    assert!(
        harness
            .query_all_by_label("No reference value")
            .next()
            .is_some()
    );
    assert!(
        harness
            .query_by_label(
                "Difference: Exact comparison unavailable per SCU · Percentage unavailable"
            )
            .is_none()
    );
    assert!(
        harness
            .query_by_label("Confirm price for commodity 42")
            .is_none()
    );
}
#[path = "manual_review/integration.rs"]
mod integration;
