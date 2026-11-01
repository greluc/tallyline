// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The single settings worker and bounded UI channels.

use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SyncSender};
use tallyline_adapter_files::api::{ConfigFile, StorageLayout};
use tallyline_ui::api::model::{BackgroundEvent, SettingsCommand};
use tallyline_ui::api::wake_up;
use tallyline_workflows::api::SettingsApplication;

type WorkerChannels = (
    SyncSender<SettingsCommand>,
    Receiver<BackgroundEvent>,
    std::thread::JoinHandle<()>,
);

pub(crate) fn start(context: egui::Context) -> io::Result<WorkerChannels> {
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(16);
    let worker = crate::executor::spawn("tallyline-settings", move || {
        let result = run(&incoming, &outgoing, &context);
        if let Err(error) = result {
            let _sent = outgoing.try_send(BackgroundEvent::Failure(error.to_string()));
            wake_up(&context);
        }
    })?;
    Ok((commands, events, worker))
}

fn deliver(
    sender: &SyncSender<BackgroundEvent>,
    context: &egui::Context,
    event: BackgroundEvent,
) -> io::Result<()> {
    sender.send(event).map_err(io::Error::other)?;
    wake_up(context);
    Ok(())
}

fn run(
    incoming: &Receiver<SettingsCommand>,
    outgoing: &SyncSender<BackgroundEvent>,
    context: &egui::Context,
) -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| io::Error::other("executable directory is missing"))?;
    let legal_event = match std::fs::read_to_string(directory.join("legal/catalogue.json")) {
        Ok(text) => match tallyline_ui::api::parse_licence_catalogue(&text) {
            Ok(catalogue) => BackgroundEvent::LicencesLoaded(catalogue),
            Err(_) => BackgroundEvent::LicencesUnavailable,
        },
        Err(_) => BackgroundEvent::LicencesUnavailable,
    };
    deliver(outgoing, context, legal_event)?;
    let layout = storage_layout()?;
    let issues = layout.self_test();
    if !issues.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            issues
                .iter()
                .map(|issue| format!("{}: {}", issue.path.display(), issue.error))
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    let store = ConfigFile::new(layout.config().join("settings.json"));
    let loaded = store.load()?;
    deliver(
        outgoing,
        context,
        BackgroundEvent::SettingsLoaded(loaded.settings.clone()),
    )?;
    if !loaded.errors.is_empty() {
        deliver(
            outgoing,
            context,
            BackgroundEvent::Failure(
                loaded
                    .errors
                    .iter()
                    .map(|issue| format!("{}: {} → {}", issue.key, issue.rejected, issue.default))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        )?;
    }
    let mut settings = SettingsApplication::new(loaded.settings, store);
    while let Ok(command) = incoming.recv() {
        let result = match command {
            SettingsCommand::ChangeTheme(theme) => settings.change_theme(theme),
            SettingsCommand::CompleteOnboarding => settings.complete_onboarding(),
        };
        match result {
            Ok(()) => deliver(
                outgoing,
                context,
                BackgroundEvent::SettingsLoaded(settings.settings().clone()),
            )?,
            Err(error) => deliver(
                outgoing,
                context,
                BackgroundEvent::Failure(error.to_string()),
            )?,
        }
    }
    Ok(())
}

fn profile_path(key: &str) -> io::Result<PathBuf> {
    std::env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing profile directory: {key}"),
            )
        })
}

fn storage_layout() -> io::Result<StorageLayout> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "executable directory missing"))?;
    if directory.join("portable.marker").is_file() {
        return Ok(StorageLayout::portable(directory));
    }
    #[cfg(windows)]
    return Ok(StorageLayout::installed_windows(
        &profile_path("APPDATA")?,
        &profile_path("LOCALAPPDATA")?,
    ));
    #[cfg(unix)]
    {
        let home = profile_path("HOME")?;
        let xdg = |key: &str, default: &str| {
            std::env::var_os(key)
                .map(PathBuf::from)
                .filter(|value| value.is_absolute())
                .unwrap_or_else(|| home.join(default))
        };
        Ok(StorageLayout::installed_linux(
            &xdg("XDG_CONFIG_HOME", ".config"),
            &xdg("XDG_DATA_HOME", ".local/share"),
            &xdg("XDG_STATE_HOME", ".local/state"),
            &xdg("XDG_CACHE_HOME", ".cache"),
        ))
    }
}
