// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded app-owned manual Report persistence worker.

use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError},
};
use std::thread::JoinHandle;
use std::time::Duration;
use tallyline_game::api::model::EnvironmentMapping;
use tallyline_reporting::api::{ReportRepository, ReportingApplication};
use tallyline_shared_kernel::api::Clock;
use tallyline_ui::api::model::{BackgroundEvent, ManualCommand};

const CONTROL_POLL: Duration = Duration::from_millis(100);
const DELIVERY_POLL: Duration = Duration::from_millis(10);

/// Owns a bounded Draft worker until shutdown joins its database connection.
pub struct ManualWorker {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<io::Result<()>>>,
}
impl ManualWorker {
    /// Starts local Draft persistence after coordinated startup migration.
    ///
    /// The app holds its writer lease until this worker is joined.
    ///
    /// # Errors
    /// Returns a worker-thread creation failure.
    pub fn start<R, C>(
        store: R,
        clock: C,
        incoming: Receiver<ManualCommand>,
        outgoing: SyncSender<BackgroundEvent>,
        mapping: impl FnMut() -> io::Result<EnvironmentMapping> + Send + 'static,
        wake: impl Fn() + Send + 'static,
    ) -> io::Result<Self>
    where
        R: ReportRepository + Send + 'static,
        C: Clock + Send + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = Arc::clone(&stop);
        let worker = crate::executor::spawn("tallyline-manual", move || {
            run(
                ReportingApplication::new(store, clock),
                incoming,
                outgoing,
                mapping,
                wake,
                &stopping,
            )
        })?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
    /// Stops accepting commands and joins the worker before the writer lease ends.
    ///
    /// # Errors
    /// Returns a worker failure or panic.
    pub fn finish(mut self) -> io::Result<()> {
        self.join()
    }
    fn join(&mut self) -> io::Result<()> {
        self.stop.store(true, Ordering::Release);
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| io::Error::other("manual Report worker terminated unexpectedly"))?,
            None => Ok(()),
        }
    }
}
impl Drop for ManualWorker {
    fn drop(&mut self) {
        if let Err(error) = self.join() {
            tracing::error!(%error,"Manual Report worker shutdown failed");
        }
    }
}
fn run<R: ReportRepository, C: Clock>(
    mut application: ReportingApplication<R, C>,
    incoming: Receiver<ManualCommand>,
    outgoing: SyncSender<BackgroundEvent>,
    mut mapping: impl FnMut() -> io::Result<EnvironmentMapping>,
    wake: impl Fn(),
    stop: &AtomicBool,
) -> io::Result<()> {
    let reports = match application.drafts() {
        Ok(reports) => reports,
        Err(error) => {
            deliver(&outgoing, BackgroundEvent::ReportFailed(error), &wake, stop)?;
            return Ok(());
        }
    };
    deliver(
        &outgoing,
        BackgroundEvent::ReportsLoaded {
            reports,
            selected: None,
        },
        &wake,
        stop,
    )?;
    while !stop.load(Ordering::Acquire) {
        let command = match incoming.recv_timeout(CONTROL_POLL) {
            Ok(command) => command,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        match mapping() {
            Ok(mapping) => application.set_mapping(mapping),
            Err(_) => {
                deliver(
                    &outgoing,
                    BackgroundEvent::ReportFailed(
                        tallyline_reporting::api::RepositoryError::Unavailable,
                    ),
                    &wake,
                    stop,
                )?;
                continue;
            }
        }
        let result = match command {
            ManualCommand::Create { key, snapshot } => {
                application.create_from_reference(key, &snapshot)
            }
            ManualCommand::Edit {
                id,
                expected,
                edit,
                snapshot,
            } => application.edit(id, expected, edit, &snapshot),
        };
        let event = match result {
            Ok(report) => match application.drafts() {
                Ok(reports) => BackgroundEvent::ReportsLoaded {
                    reports,
                    selected: Some(report.id()),
                },
                Err(error) => BackgroundEvent::ReportFailed(error),
            },
            Err(error) => BackgroundEvent::ReportFailed(error),
        };
        deliver(&outgoing, event, &wake, stop)?;
    }
    Ok(())
}
fn deliver(
    sender: &SyncSender<BackgroundEvent>,
    mut event: BackgroundEvent,
    wake: &impl Fn(),
    stop: &AtomicBool,
) -> io::Result<()> {
    loop {
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        match sender.try_send(event) {
            Ok(()) => {
                wake();
                return Ok(());
            }
            Err(TrySendError::Full(unsent)) => {
                event = unsent;
                std::thread::sleep(DELIVERY_POLL);
            }
            Err(TrySendError::Disconnected(_)) => return Ok(()),
        }
    }
}
