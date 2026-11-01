// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Owned shell state and worker result handling.

use crate::api::DialogPresenter;
use tallyline_workflows::api::model::{AppSettings, Theme};

/// A primary shell view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    /// First-launch legal and setup entry screen.
    #[default]
    Onboarding,
    /// The main workspace and its empty capture state.
    Home,
    /// Application preferences.
    Settings,
}

/// A worker result that never requests a dialog or window command.
#[derive(Debug)]
pub enum BackgroundEvent {
    /// An actionable failure that stays visible across view changes.
    Failure(String),
    /// Preferences loaded or successfully committed by the file worker.
    SettingsLoaded(AppSettings),
    /// The validated offline catalogue, without requesting a dialog.
    LicencesLoaded(crate::api::model::LicenceCatalogue),
    /// The bundled catalogue is missing, unreadable or incomplete.
    LicencesUnavailable,
}

/// Owned shell state shared by drawn and windowless tests.
#[derive(Default)]
pub struct ShellState {
    view: View,
    notice: Option<String>,
    dialogs: DialogPresenter,
    settings: AppSettings,
    busy: bool,
    ready: bool,
}

impl ShellState {
    /// Returns whether startup established a writable settings store.
    #[must_use]
    pub const fn ready(&self) -> bool {
        self.ready
    }

    /// Opens About in direct response to a user action.
    pub fn open_about(&mut self) {
        self.dialogs.open_about();
    }

    /// Draws dialogs owned by the presenter.
    pub fn show_dialogs(
        &mut self,
        context: &egui::Context,
        catalogue: &crate::api::Catalogue,
        unit: &crate::api::FanKitUnit,
    ) {
        self.dialogs.show(context, catalogue, unit);
    }
    /// Returns the preferences confirmed by the settings worker.
    #[must_use]
    pub const fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Returns whether a settings update is waiting for its worker.
    #[must_use]
    pub const fn busy(&self) -> bool {
        self.busy
    }

    /// Converts a direct user action into a bounded worker command.
    pub fn act(&mut self, action: UserAction) -> Option<SettingsCommand> {
        if self.busy || !self.ready {
            return None;
        }
        self.busy = true;
        Some(match action {
            UserAction::ChooseTheme(theme) => SettingsCommand::ChangeTheme(theme),
            UserAction::ContinueOnboarding => SettingsCommand::CompleteOnboarding,
        })
    }
    /// Applies a worker result without changing the view or window state.
    pub fn receive(&mut self, event: BackgroundEvent) {
        match event {
            BackgroundEvent::Failure(notice) => {
                self.notice = Some(notice);
                self.busy = false;
            }
            BackgroundEvent::LicencesLoaded(catalogue) => {
                self.dialogs.set_licences(Some(catalogue))
            }
            BackgroundEvent::LicencesUnavailable => self.dialogs.set_licences(None),
            BackgroundEvent::SettingsLoaded(settings) => {
                if self.view == View::Onboarding && settings.onboarding_complete() {
                    self.view = View::Home;
                }
                self.settings = settings;
                self.ready = true;
                self.notice = None;
                self.busy = false;
            }
        }
    }

    /// Returns the persistent action notice.
    #[must_use]
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Returns the active primary view.
    #[must_use]
    pub const fn view(&self) -> View {
        self.view
    }

    /// Returns user-driven dialog state.
    #[must_use]
    pub const fn dialogs(&self) -> &DialogPresenter {
        &self.dialogs
    }

    /// Returns to the workspace, or to unfinished onboarding.
    pub fn leave_settings(&mut self) {
        self.view = if self.settings.onboarding_complete() {
            View::Home
        } else {
            View::Onboarding
        };
    }

    /// Opens preferences after a user action.
    pub fn show_settings(&mut self) {
        self.view = View::Settings;
    }
}

/// A direct user request that requires durable preferences.
pub enum UserAction {
    /// Select a colour theme.
    ChooseTheme(Theme),
    /// Acknowledge the first-launch screen.
    ContinueOnboarding,
}

/// A settings operation handled outside the UI thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsCommand {
    /// Save and apply a colour theme.
    ChangeTheme(Theme),
    /// Save acknowledgement of the onboarding screen.
    CompleteOnboarding,
}
