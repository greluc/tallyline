// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawn immutable queue modes and cancellation limits.

use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use tallyline_shared_kernel::api::model::{
    CommodityId, ReportId, RowId, SubmissionMode, TerminalId, Timestamp, TradeSide, UexEnvironment,
};
use tallyline_submission::api::model::{
    JobId, PayloadHash, SubmissionHeader, SubmissionJob, SubmissionRequest, SubmissionRow,
};
use tallyline_ui::api::model::{QueueCommand, SubmissionQueue};
use tallyline_ui::api::{Catalogue, show_submission_queue};
mod submission_support;
use submission_support::font_files;
fn queued(id: u64, mode: SubmissionMode) -> Option<SubmissionJob> {
    SubmissionJob::queued(
        JobId(id),
        SubmissionRequest::new(
            SubmissionHeader {
                report: ReportId(id + 100),
                report_version: 3,
                account: "test-account".to_owned(),
                mode,
                terminal: TerminalId(89),
                side: TradeSide::Sell,
                environment: UexEnvironment::Live,
                game_version: "4.10.2".to_owned(),
                observed_at: Timestamp::from_seconds(100),
            },
            vec![
                SubmissionRow::new(
                    RowId(1),
                    CommodityId(7),
                    "120".parse().ok()?,
                    "0".parse().ok()?,
                    1,
                )
                .ok()?,
            ],
        )
        .ok()?,
    )
    .ok()
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_sub_2_r_sub_8_r_ui_16__drawn_queue_keeps_mode_account_and_sending_cancel_limit_visible() {
    let first = queued(1, SubmissionMode::DryRun).unwrap();
    let sending = queued(2, SubmissionMode::Production)
        .unwrap()
        .begin(Timestamp::from_seconds(110), PayloadHash::new([1; 32]))
        .unwrap();
    let mut state = SubmissionQueue::default();
    let returned = queued(3, SubmissionMode::DryRun)
        .unwrap()
        .return_for_review(tallyline_submission::api::model::SendReturnReason::GameVersionChanged)
        .unwrap();
    let held = queued(4, SubmissionMode::Production)
        .unwrap()
        .begin(Timestamp::from_seconds(110), PayloadHash::new([1; 32]))
        .unwrap()
        .complete(
            tallyline_submission::api::model::SubmissionResult::NotProcessed(
                tallyline_submission::api::model::NotProcessed::ConnectFailure,
            ),
            Timestamp::from_seconds(115),
        )
        .unwrap();
    state.received(vec![first, sending, returned, held]);
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 1000.0))
        .build_ui_state(
            move |ui, state: &mut (SubmissionQueue, Vec<QueueCommand>)| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                if let Some(command) = show_submission_queue(
                    ui,
                    &mut state.0,
                    &catalogue,
                    tallyline_egui::Theme::Light,
                ) {
                    state.1.push(command);
                }
            },
            (state, Vec::new()),
        );
    harness.run();
    harness.get_by_label("Report 101 · Submission 1 · test-account");
    assert_eq!(harness.query_all_by_label("Dry run").count(), 2);
    assert_eq!(harness.query_all_by_label("Production").count(), 2);
    harness.get_by_label("Sending");
    harness.get_by_label("The connection failed before sending. Cancel this submission to review and release the Report again.");
    harness.get_by_label(
        "The game version changed. Review the captured version before releasing the Report again.",
    );
    assert!(
        harness
            .get_by_label("Cancel submission 2")
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label("Cancel submission 1").focus();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().1,
        vec![QueueCommand::Cancel {
            id: JobId(1),
            expected_revision: 1
        }]
    );
    assert!(
        harness
            .get_by_label("Cancel submission 1")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        harness
            .output()
            .viewport_output
            .values()
            .all(|output| output.commands.is_empty())
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_sub_9__stale_queue_results_cannot_reopen_sending_or_unknown_for_cancellation() {
    let first = queued(1, SubmissionMode::DryRun).unwrap();
    let sending = first
        .begin(Timestamp::from_seconds(110), PayloadHash::new([1; 32]))
        .unwrap();
    let mut state = SubmissionQueue::default();
    state.received(vec![first.clone()]);
    assert!(state.request_cancel(JobId(1)));
    state.received(vec![sending.clone()]);
    assert!(state.take_command().is_none());
    state.received(vec![first]);
    assert_eq!(state.jobs()[0], sending);
    assert!(!state.request_cancel(JobId(1)));
    let unknown = sending.recover(Timestamp::from_seconds(120)).unwrap();
    state.received(vec![unknown.clone(), sending]);
    assert_eq!(state.jobs()[0], unknown);
    assert!(!state.request_cancel(JobId(1)));
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_sub_9_r_ui_16__failed_cancel_retains_committed_state_and_only_exact_feedback_unlocks_the_action()
 {
    use tallyline_submission::api::model::{SubmissionApplicationError, SubmissionRepositoryError};
    let first = queued(1, SubmissionMode::DryRun).unwrap();
    let mut state = SubmissionQueue::default();
    state.received(vec![first.clone()]);
    assert!(state.request_cancel(JobId(1)));
    assert_eq!(
        state.take_command(),
        Some(QueueCommand::Cancel {
            id: JobId(1),
            expected_revision: 1
        })
    );
    let failure = SubmissionApplicationError::Repository(SubmissionRepositoryError::Unavailable);
    state.failed(JobId(2), 1, failure);
    assert!(!state.can_cancel(JobId(1)));
    state.failed(JobId(1), 2, failure);
    assert!(!state.can_cancel(JobId(1)));
    state.failed(JobId(1), 1, failure);
    assert!(state.can_cancel(JobId(1)));
    assert_eq!(state.jobs(), &[first]);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_sub_9__queue_publishes_committed_receipt_mapping_only_for_its_exact_successful_job() {
    use tallyline_shared_kernel::api::model::UexReportId;
    use tallyline_submission::api::model::{
        SubmissionResult, VerifiedReceiptMapping, VerifiedUexReport,
    };
    let queued = queued(4, SubmissionMode::Production).unwrap();
    let accepted = queued
        .begin(Timestamp::from_seconds(110), PayloadHash::new([1; 32]))
        .unwrap()
        .complete(
            SubmissionResult::ProductionAccepted(vec![
                UexReportId::new(42, Timestamp::from_seconds(100)).unwrap(),
            ]),
            Timestamp::from_seconds(115),
        )
        .unwrap();
    let receipt = VerifiedUexReport::from_lookup(
        42,
        TerminalId(89),
        CommodityId(7),
        Timestamp::from_seconds(100),
        accepted.request(),
    )
    .unwrap();
    let mapping = VerifiedReceiptMapping::new(&accepted, vec![receipt]).unwrap();
    let mut state = SubmissionQueue::default();
    state.received(vec![queued.clone()]);
    assert!(!state.received_mapping(mapping.clone()));
    assert!(state.verified_receipts(JobId(4)).is_none());
    state.received(vec![accepted.clone()]);
    assert!(state.received_mapping(mapping));
    assert_eq!(state.verified_receipts(JobId(4)).unwrap()[0].id(), 42);
    state.received(vec![queued]);
    assert_eq!(state.jobs()[0], accepted);
    assert_eq!(state.verified_receipts(JobId(4)).unwrap()[0].id(), 42);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_sub_2_r_sub_8_r_ui_1__drawn_waits_show_exact_cooldown_budget_and_account_reason_and_ignore_stale_sending_views()
 {
    use tallyline_submission::api::model::{SubmissionWait, SubmissionWaitReason};
    let first = queued(11, SubmissionMode::DryRun).unwrap();
    let second = queued(12, SubmissionMode::Production).unwrap();
    let third = queued(13, SubmissionMode::Production).unwrap();
    let mut state = SubmissionQueue::default();
    state.received(vec![first.clone(), second.clone(), third.clone()]);
    state.received_waiting(vec![
        SubmissionWait {
            job: first.id(),
            revision: first.revision(),
            reason: SubmissionWaitReason::CooldownUntil(Timestamp::from_seconds(300)),
            next_check: None,
        },
        SubmissionWait {
            job: second.id(),
            revision: second.revision(),
            reason: SubmissionWaitReason::RowBudget,
            next_check: None,
        },
        SubmissionWait {
            job: third.id(),
            revision: third.revision(),
            reason: SubmissionWaitReason::AccountChanged,
            next_check: None,
        },
    ]);
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 1600.0))
        .build_ui_state(
            move |ui, state: &mut SubmissionQueue| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_submission_queue(ui, state, &catalogue, tallyline_egui::Theme::Light);
            },
            state,
        );
    harness.run();
    harness.get_by_label("Waiting for cooldown until 1970-01-01 00:05:00 UTC");
    harness.get_by_label("Waiting for the complete Report to fit the 1,000-row production window.");
    harness.get_by_label("Paused: another account is active. Keep pending for the released account or cancel and review.");
    let sending = first
        .begin(Timestamp::from_seconds(301), PayloadHash::new([1; 32]))
        .unwrap();
    harness.state_mut().received(vec![sending]);
    harness.state_mut().received_waiting(vec![SubmissionWait {
        job: first.id(),
        revision: first.revision(),
        reason: SubmissionWaitReason::CooldownUntil(Timestamp::from_seconds(300)),
        next_check: None,
    }]);
    harness.run();
    assert!(
        harness
            .query_by_label("Waiting for cooldown until 1970-01-01 00:05:00 UTC")
            .is_none()
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_sub_12_r_ui_16__account_switch_offers_keep_or_review_without_rebinding_jobs_or_opening_a_dialog()
 {
    use tallyline_submission::api::model::{AccountState, Persistence};
    use tallyline_ui::api::model::{BackgroundEvent, SubmissionUpdate, View};
    use tallyline_ui::api::{ShellState, show_submission_account_choice};
    let job = queued(31, SubmissionMode::Production).unwrap();
    let mut shell = ShellState::default();
    shell.receive(BackgroundEvent::SettingsLoaded(
        tallyline_workflows::api::model::AppSettings::default(),
    ));
    let account = |name: &str, at| BackgroundEvent::CredentialsUpdated {
        account: AccountState::Validated {
            username: name.to_owned(),
            validated_at: Timestamp::from_seconds(at),
            persistence: Persistence::Session,
        },
        failure: None,
        available_token: true,
        fallback_available: false,
    };
    shell.receive(account("test-account", 1000));
    shell.receive(BackgroundEvent::SubmissionUpdated(Box::new(
        SubmissionUpdate {
            jobs: vec![job.clone()],
            waiting: vec![],
            mappings: vec![],
            reports: vec![],
            feedback: None,
            background_feedback: None,
            completed: false,
        },
    )));
    shell.show_settings();
    shell.receive(account("other-account", 1001));
    assert_eq!(shell.view(), View::Settings);
    assert!(!shell.dialogs().about_open());
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(font_files().unwrap());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 500.0))
        .build_ui_state(
            move |ui, state: &mut ShellState| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_submission_account_choice(ui, state, &catalogue);
            },
            shell,
        );
    harness.run();
    harness.get_by_label("Queued submissions belong to another account. Keep them for their released accounts or review and cancel them explicitly.");
    harness.get_by_label("Review queued submissions").click();
    harness.run();
    assert_eq!(harness.state().view(), View::Queue);
    harness
        .get_by_label("Keep pending for their released accounts")
        .click();
    harness.run();
    assert_eq!(harness.state().submission_jobs(), &[job]);
    assert!(harness.state_mut().take_submission_command().is_none());
    harness.state_mut().receive(account("other-account", 1002));
    harness.run();
    assert!(
        harness
            .query_by_label("Keep pending for their released accounts")
            .is_none()
    );
    assert_eq!(harness.state().view(), View::Queue);
    assert!(!harness.state().dialogs().about_open());
}
