// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Explicit second-launch window commands remain separate from background events.

use tallyline_ui::api::{apply_window_command, model::WindowCommand};

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_10__explicit_activation_restores_and_requests_native_attention() {
    let context = egui::Context::default();
    let mut output = context.run_ui(egui::RawInput::default(), |ui| {
        apply_window_command(ui.ctx(), WindowCommand::Activate);
    });
    output.textures_delta.clear();
    let commands: Vec<_> = output
        .viewport_output
        .values()
        .flat_map(|viewport| viewport.commands.iter())
        .collect();
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, egui::ViewportCommand::Minimized(false)))
    );
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, egui::ViewportCommand::Focus))
    );
    assert!(commands.iter().any(|command| matches!(
        command,
        egui::ViewportCommand::RequestUserAttention(egui::UserAttentionType::Informational)
    )));
}
