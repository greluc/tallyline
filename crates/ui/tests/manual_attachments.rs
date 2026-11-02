// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Critical drawn private-preview and current-Report contracts without a native window.

use egui_kittest::{Harness, kittest::Queryable};
use tallyline_game::api::model::GameEnvironment;
use tallyline_reporting::api::AttachmentEdit;
use tallyline_reporting::api::model::{
    AttachmentBinding, AttachmentRegion, AttachmentTime, ImageHash, Report, ReportId, ReportKey,
};
use tallyline_shared_kernel::api::model::{ImageRaster, TerminalId, Timestamp, TradeSide};
use tallyline_submission::api::model::{AccountState, Persistence, ScreenshotPolicy};
use tallyline_ui::api::model::{BackgroundEvent, ManualAttachmentAction, ManualCommand};
use tallyline_ui::api::{Catalogue, ShellState, show_manual_attachment_editor};
use tallyline_workflows::api::{ManualAttachmentBlock, ManualAttachmentPreview};

fn selected() -> Result<(ShellState, Report, ManualAttachmentPreview), String> {
    let mut report = Report::manual(
        ReportId(1),
        ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live)
            .map_err(|error| format!("{error:?}"))?,
        Timestamp::from_seconds(100),
        Some("4.10.2"),
    )
    .map_err(|error| format!("{error:?}"))?;
    let binding = AttachmentBinding::new(
        ImageHash::new([1; 32]),
        AttachmentRegion::new(0, 10, 4, 2).map_err(|error| format!("{error:?}"))?,
    );
    report
        .edit_attachment(AttachmentEdit::Select(
            binding,
            AttachmentTime::ImportedFile(Timestamp::from_seconds(90)),
        ))
        .map_err(|error| format!("{error:?}"))?;
    let preview = ManualAttachmentPreview::new(
        binding,
        AttachmentTime::ImportedFile(Timestamp::from_seconds(90)),
        ImageRaster::new(4, 2, vec![0xff00_c800; 8]).map_err(|error| format!("{error:?}"))?,
    )
    .map_err(|error| format!("{error:?}"))?;
    let mut state = ShellState::default();
    state.receive(BackgroundEvent::ReportsLoaded {
        reports: vec![report.clone()],
        selected: Some(report.id()),
    });
    Ok((state, report, preview))
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_5_r_sub_7__changed_content_geometry_and_removal_reject_delayed_preview_results() {
    for action in [
        AttachmentEdit::Select(
            AttachmentBinding::new(
                ImageHash::new([2; 32]),
                AttachmentRegion::new(0, 10, 4, 2).unwrap(),
            ),
            AttachmentTime::ImportedFile(Timestamp::from_seconds(90)),
        ),
        AttachmentEdit::Select(
            AttachmentBinding::new(
                ImageHash::new([1; 32]),
                AttachmentRegion::new(0, 11, 4, 2).unwrap(),
            ),
            AttachmentTime::ImportedFile(Timestamp::from_seconds(90)),
        ),
        AttachmentEdit::Remove,
    ] {
        let (mut state, mut report, preview) = selected().unwrap();
        state.receive(BackgroundEvent::AttachmentPreview {
            id: report.id(),
            expected: report.version(),
            preview: preview.clone(),
        });
        assert!(state.manual().attachment_preview().is_some());
        let expected = report.version();
        report.edit_attachment(action).unwrap();
        state.receive(BackgroundEvent::ReportsLoaded {
            reports: vec![report.clone()],
            selected: Some(report.id()),
        });
        state.receive(BackgroundEvent::AttachmentPreview {
            id: report.id(),
            expected,
            preview,
        });
        assert!(state.manual().attachment_preview().is_none());
    }
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_5_r_sub_7__dated_requirement_and_full_size_preview_have_explicit_clear_and_privacy_actions()
 {
    let (mut state, report, preview) = selected().unwrap();
    let account = AccountState::Validated {
        username: "dummy-account".to_owned(),
        validated_at: Timestamp::from_seconds(100),
        persistence: Persistence::Session,
    };
    state.receive(BackgroundEvent::AttachmentPreview {
        id: report.id(),
        expected: report.version(),
        preview,
    });
    state.receive(BackgroundEvent::AttachmentStatus {
        id: report.id(),
        expected: report.version(),
        account: account.clone(),
        policy: Ok(ScreenshotPolicy::Required {
            observed_at: Timestamp::from_seconds(0),
        }),
        readiness: Err(ManualAttachmentBlock::PreviewUnconfirmed),
    });
    let catalogue = Catalogue::english().unwrap();
    let mut fonts = Some(font_files());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 1000.0))
        .build_ui_state(
            move |ui, state| {
                if let Some(fonts) = fonts.take() {
                    tallyline_ui::api::configure_fonts(ui.ctx(), fonts);
                }
                show_manual_attachment_editor(ui, state, &catalogue, &account);
            },
            state,
        );
    harness.run();
    assert!(
        harness
            .query_by_label("Screenshot required since 1970-01-01 00:00:00 UTC")
            .is_some()
    );
    harness.get_by_label("Try without screenshot").click();
    harness.run();
    assert!(
        matches!(harness.state_mut().manual_mut().take_command(), Some(ManualCommand::Attachment(command)) if matches!(&command.action, ManualAttachmentAction::TryWithoutScreenshot(AccountState::Validated { username, .. }) if username == "dummy-account"))
    );
    harness
        .state_mut()
        .receive(BackgroundEvent::AttachmentStatus {
            id: report.id(),
            expected: report.version(),
            account: AccountState::Validated {
                username: "dummy-account".to_owned(),
                validated_at: Timestamp::from_seconds(100),
                persistence: Persistence::Session,
            },
            policy: Ok(ScreenshotPolicy::Unknown),
            readiness: Err(ManualAttachmentBlock::PreviewUnconfirmed),
        });
    harness.run();
    assert!(
        harness
            .query_by_label("Screenshot required since 1970-01-01 00:00:00 UTC")
            .is_none()
    );
    harness
        .get_by_label("Contains no balance or personal data")
        .click();
    harness.run();
    assert!(
        matches!(harness.state_mut().manual_mut().take_command(), Some(ManualCommand::Attachment(command)) if matches!(command.action, ManualAttachmentAction::ConfirmPreview(_)))
    );
}

fn font_files() -> tallyline_ui::api::model::FontFiles {
    tallyline_ui::api::model::FontFiles {
        plex_regular: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-Regular.ttf").to_vec(),
        plex_semibold: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-SemiBold.ttf").to_vec(),
        plex_bold: include_bytes!("../assets/fonts/ibm-plex/IBMPlexSans-Bold.ttf").to_vec(),
        mono_regular: include_bytes!("../assets/fonts/jetbrains-mono/JetBrainsMonoNL-Regular.ttf")
            .to_vec(),
        mono_bold: include_bytes!("../assets/fonts/jetbrains-mono/JetBrainsMonoNL-Bold.ttf")
            .to_vec(),
    }
}
