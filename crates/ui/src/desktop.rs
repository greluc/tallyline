// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native frame rendering and bounded UI message delivery.

use crate::api::model::{BackgroundEvent, SettingsCommand};
use crate::api::{Catalogue, FanKitUnit, ShellState, show_shell};
use std::sync::mpsc::{Receiver, SyncSender};
use tallyline_egui::{Density, Motion, Theme as UiTheme, apply_theme};

/// The native rendering host with owned view state and injected worker channels.
pub struct DesktopShell {
    state: ShellState,
    catalogue: Catalogue,
    unit: FanKitUnit,
    dark_unit: FanKitUnit,
    commands: Option<SyncSender<SettingsCommand>>,
    events: Receiver<BackgroundEvent>,
    theme: Option<UiTheme>,
}

impl DesktopShell {
    /// Creates the view host from predecoded artwork and bounded worker channels.
    #[must_use]
    pub fn new(
        context: &egui::Context,
        catalogue: Catalogue,
        light_pixels: egui::ColorImage,
        dark_pixels: egui::ColorImage,
        commands: SyncSender<SettingsCommand>,
        events: Receiver<BackgroundEvent>,
    ) -> Self {
        Self {
            state: ShellState::default(),
            unit: FanKitUnit::new(context, &catalogue, light_pixels),
            dark_unit: FanKitUnit::new(context, &catalogue, dark_pixels),
            catalogue,
            commands: Some(commands),
            events,
            theme: None,
        }
    }
}
impl eframe::App for DesktopShell {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        for event in self.events.try_iter() {
            self.state.receive(event);
        }
        let theme =
            crate::api::resolve_theme(self.state.settings().theme(), ui.ctx().system_theme());
        if self.theme != Some(theme) {
            ui.ctx().set_theme(match theme {
                UiTheme::Light => egui::ThemePreference::Light,
                UiTheme::Dark | UiTheme::HighContrast => egui::ThemePreference::Dark,
            });
            apply_theme(ui.ctx(), theme, Density::Comfortable, Motion::Reduced);
            self.theme = Some(theme);
        }
        let unit = if theme == UiTheme::Light {
            &self.unit
        } else {
            &self.dark_unit
        };
        ui.set_style(ui.ctx().global_style());
        let commands = egui::CentralPanel::default()
            .show(ui, |ui| {
                show_shell(ui, &mut self.state, &self.catalogue, unit)
            })
            .inner;
        for command in commands {
            if let Some(sender) = &self.commands
                && sender.try_send(command).is_err()
            {
                self.state.receive(BackgroundEvent::Failure(
                    self.catalogue
                        .text("settings.worker_unavailable")
                        .to_owned(),
                ));
            }
        }
    }
}

impl Drop for DesktopShell {
    fn drop(&mut self) {
        drop(self.commands.take());
    }
}
