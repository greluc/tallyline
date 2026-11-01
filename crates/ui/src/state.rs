// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Owned shell state and worker result handling.

use crate::api::DialogPresenter;
use crate::game_witness::GameWitnessView;
use tallyline_game::api::model::WitnessMode;
use tallyline_workflows::api::model::{AppSettings, Theme};

/// A primary shell view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    /// First-launch legal and setup entry screen.
    #[default]
    Onboarding,
    /// The main workspace and its empty capture state.
    Home,
    /// Cached environment-specific terminal and commodity browsing.
    Reference,
    /// Persistent keyboard-driven manual Report entry.
    Manual,
    /// Application preferences.
    Settings,
}

/// A worker result that never requests a dialog or window command.
#[derive(Debug)]
pub enum BackgroundEvent {
    /// Historical non-JSON metadata, without changing current cache or window state.
    ReferenceDiagnostic(crate::api::model::ReferenceDiagnostic),
    /// Durable Drafts loaded after storage commits, without changing focus or view.
    ReportsLoaded {
        /// Most recently edited Drafts first.
        reports: Vec<tallyline_reporting::api::model::Report>,
        /// Draft explicitly created or edited by this user action.
        selected: Option<tallyline_reporting::api::model::ReportId>,
    },
    /// A manual Report operation failed without losing the displayed input.
    ReportFailed(tallyline_reporting::api::RepositoryError),
    /// A passive refresh starts without replacing cached records.
    ReferenceRefreshStarted,
    /// Environment-specific cache state and its injected clock observation.
    ReferenceUpdated {
        /// LIVE records.
        live: tallyline_reference_data::api::model::ReferenceSnapshot,
        /// PTU records with no borrowed LIVE priors.
        ptu: tallyline_reference_data::api::model::ReferenceSnapshot,
        /// The worker clock when these snapshots were published.
        observed_at: tallyline_shared_kernel::api::model::Timestamp,
    },
    /// Passive release metadata result with no window command.
    UpdateChecked(tallyline_workflows::api::model::UpdateStatus),
    /// Local witness settings and sanitised facts from the file worker.
    GameWitnessLoaded(GameWitnessView),
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
    manual: crate::manual_editor::ManualEditor,
    references: crate::reference_browser::ReferenceBrowser,
    notice: Option<String>,
    reference_notice: Option<String>,
    dialogs: DialogPresenter,
    settings: AppSettings,
    busy: bool,
    ready: bool,
    game_witness: GameWitnessView,
    game_channel_draft: String,
    update_status: tallyline_workflows::api::model::UpdateStatus,
}

impl ShellState {
    /// Returns the user-editable local channel selection.
    pub fn game_channel_draft(&mut self) -> &mut String {
        &mut self.game_channel_draft
    }
    /// Returns the last durable local witness observation.
    #[must_use]
    pub const fn game_witness(&self) -> &GameWitnessView {
        &self.game_witness
    }

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
            UserAction::ChangeEnvironmentMapping(channel, environment) => {
                SettingsCommand::ChangeEnvironmentMapping(channel, environment)
            }
            UserAction::DismissUpdate => SettingsCommand::DismissUpdate,
            UserAction::ChangeUpdateChecks(checks) => SettingsCommand::ChangeUpdateChecks(checks),
            UserAction::ChooseTheme(theme) => SettingsCommand::ChangeTheme(theme),
            UserAction::ChooseGameChannel(channel) => SettingsCommand::ChangeGameChannel(channel),
            UserAction::RefreshWitness => SettingsCommand::RefreshWitness,
            UserAction::ChangeWitnessMode(mode) => SettingsCommand::ChangeWitnessMode(mode),
            UserAction::ContinueOnboarding => SettingsCommand::CompleteOnboarding,
        })
    }
    /// Applies a worker result without changing the view or window state.
    pub fn receive(&mut self, event: BackgroundEvent) {
        match event {
            BackgroundEvent::ReferenceDiagnostic(diagnostic) => {
                self.references.report_diagnostic(diagnostic)
            }
            BackgroundEvent::ReportsLoaded { reports, selected } => {
                self.manual.loaded(reports, selected)
            }
            BackgroundEvent::ReportFailed(error) => self.manual.failed(error),
            BackgroundEvent::ReferenceRefreshStarted => self.references.started(),
            BackgroundEvent::ReferenceUpdated {
                live,
                ptu,
                observed_at,
            } => {
                self.references.updated(live, ptu, observed_at);
                self.reference_notice = None;
            }
            BackgroundEvent::UpdateChecked(status) => {
                if self.settings.update_checks()
                    == tallyline_workflows::api::model::UpdateChecks::Enabled
                {
                    self.update_status = status;
                }
            }
            BackgroundEvent::GameWitnessLoaded(view) => {
                self.game_channel_draft = view.channel_directory.clone().unwrap_or_default();
                self.game_witness = view;
                self.busy = false;
            }
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
                if settings.update_checks()
                    == tallyline_workflows::api::model::UpdateChecks::Disabled
                {
                    self.update_status = tallyline_workflows::api::model::UpdateStatus::NotChecked;
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
        self.notice.as_deref().or(self.reference_notice.as_deref())
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
    /// Persist an independent UEX environment for one game channel.
    ChangeEnvironmentMapping(
        tallyline_game::api::model::GameEnvironment,
        tallyline_shared_kernel::api::model::UexEnvironment,
    ),
    /// Persist dismissal of the available release.
    DismissUpdate,
    /// Change permission for daily release metadata checks.
    ChangeUpdateChecks(tallyline_workflows::api::model::UpdateChecks),
    /// Choose a channel directory and revoke the previous opt-in.
    ChooseGameChannel(String),
    /// Refresh only after an explicit user request.
    RefreshWitness,
    /// Enable or disable read-only witness access.
    ChangeWitnessMode(WitnessMode),
    /// Select a colour theme.
    ChooseTheme(Theme),
    /// Acknowledge the first-launch screen.
    ContinueOnboarding,
}

/// A settings operation handled outside the UI thread.
#[derive(Clone, PartialEq, Eq)]
pub enum SettingsCommand {
    /// Persist an independent UEX environment for one game channel.
    ChangeEnvironmentMapping(
        tallyline_game::api::model::GameEnvironment,
        tallyline_shared_kernel::api::model::UexEnvironment,
    ),
    /// Save the explicit release notice dismissal.
    DismissUpdate,
    /// Persist release-check permission.
    ChangeUpdateChecks(tallyline_workflows::api::model::UpdateChecks),
    /// Persist a channel directory and disable witness access.
    ChangeGameChannel(String),
    /// Read sanitised facts under the durable opt-in preference.
    RefreshWitness,
    /// Persist explicit read-only witness access.
    ChangeWitnessMode(WitnessMode),
    /// Save and apply a colour theme.
    ChangeTheme(Theme),
    /// Save acknowledgement of the onboarding screen.
    CompleteOnboarding,
}

impl std::fmt::Debug for SettingsCommand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChangeEnvironmentMapping(channel, environment) => formatter
                .debug_tuple("ChangeEnvironmentMapping")
                .field(channel)
                .field(environment)
                .finish(),
            Self::DismissUpdate => formatter.write_str("DismissUpdate"),
            Self::ChangeUpdateChecks(checks) => formatter
                .debug_tuple("ChangeUpdateChecks")
                .field(checks)
                .finish(),
            Self::ChangeGameChannel(_) => {
                formatter.write_str("ChangeGameChannel(<local selection>)")
            }
            Self::RefreshWitness => formatter.write_str("RefreshWitness"),
            Self::ChangeWitnessMode(mode) => formatter
                .debug_tuple("ChangeWitnessMode")
                .field(mode)
                .finish(),
            Self::ChangeTheme(theme) => formatter.debug_tuple("ChangeTheme").field(theme).finish(),
            Self::CompleteOnboarding => formatter.write_str("CompleteOnboarding"),
        }
    }
}

impl ShellState {
    /// Returns the most recent passive update status.
    #[must_use]
    pub const fn update_status(&self) -> &tallyline_workflows::api::model::UpdateStatus {
        &self.update_status
    }
}

mod reference;

impl ShellState {
    /// Returns manual Draft editing state.
    #[must_use]
    pub const fn manual(&self) -> &crate::api::model::ManualEditor {
        &self.manual
    }
    /// Returns mutable manual input and command state.
    pub fn manual_mut(&mut self) -> &mut crate::api::model::ManualEditor {
        &mut self.manual
    }
    /// Opens manual entry in direct response to a user action.
    pub fn show_manual(&mut self) {
        self.view = View::Manual;
    }
}
