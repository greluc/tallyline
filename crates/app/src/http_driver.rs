// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded shared HTTP runtime whose lifetime covers every pooled connection.

use crate::http_client::HttpClient;
use crate::http_protocol::{Command, HttpBackend};
use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::instrument::WithSubscriber;

const IO_THREADS: usize = 2;
const MAX_ACTIVE: usize = 2;
const MAX_BLOCKING_THREADS: usize = 2;
const MAX_SUBMISSIONS: usize = 1;
const REQUEST_CAPACITY: usize = 2;
const CONTROL_POLL: Duration = Duration::from_millis(100);

/// Owns the sole HTTP backend and joins its bounded shared runtime after client workers stop.
pub struct HttpDriver {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<io::Result<()>>>,
    processor: Option<crate::response_worker::ResponseWorker>,
}
impl HttpDriver {
    /// Starts one backend factory on a private thread and exactly two I/O runtime threads.
    ///
    /// # Errors
    /// Returns thread creation failure; asynchronous startup failures surface through the clients and finish.
    pub fn start<B, E>(
        factory: impl FnOnce() -> Result<B, E> + Send + 'static,
    ) -> io::Result<(Self, HttpClient)>
    where
        B: HttpBackend + 'static,
    {
        Self::start_configured(factory, None)
    }
    /// Shares one server-time observation between response handling and body preparation.
    ///
    /// # Errors
    /// Returns thread creation failure; backend startup failure closes its clients.
    pub fn start_with_server_time<B, E>(
        factory: impl FnOnce() -> Result<B, E> + Send + 'static,
        server_time: tallyline_adapter_uex::api::model::ServerTimeTracker,
    ) -> io::Result<(Self, HttpClient)>
    where
        B: HttpBackend + 'static,
    {
        Self::start_configured(factory, Some(server_time))
    }
    /// Owns and joins the bounded below-normal response CPU worker alongside the shared I/O runtime.
    ///
    /// # Errors
    /// Returns CPU or I/O thread creation failure; backend failures close their clients.
    pub fn start_with_processing<B, E>(
        factory: impl FnOnce(tallyline_adapter_uex::api::ResponseProcessor) -> Result<B, E>
        + Send
        + 'static,
        server_time: Option<tallyline_adapter_uex::api::model::ServerTimeTracker>,
    ) -> io::Result<(Self, HttpClient)>
    where
        B: HttpBackend + 'static,
    {
        let (worker, processor) = crate::response_worker::ResponseWorker::start()?;
        let (mut driver, client) = Self::start_configured(move || factory(processor), server_time)?;
        driver.processor = Some(worker);
        Ok((driver, client))
    }
    fn start_configured<B, E>(
        factory: impl FnOnce() -> Result<B, E> + Send + 'static,
        server_time: Option<tallyline_adapter_uex::api::model::ServerTimeTracker>,
    ) -> io::Result<(Self, HttpClient)>
    where
        B: HttpBackend + 'static,
    {
        let (requests, incoming) = mpsc::channel(REQUEST_CAPACITY);
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = Arc::clone(&stop);
        let worker = crate::executor::spawn("tallyline-http", move || {
            tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
                run(factory, incoming, &stopping)
            })
        })?;
        Ok((
            Self {
                stop,
                worker: Some(worker),
                processor: None,
            },
            HttpClient {
                requests,
                server_time,
            },
        ))
    }
    /// Stops the driver after all application clients have finished and joins every I/O thread.
    ///
    /// # Errors
    /// Returns a closed backend startup or worker failure.
    pub fn finish(mut self) -> io::Result<()> {
        self.join()
    }
    fn join(&mut self) -> io::Result<()> {
        self.stop.store(true, Ordering::Release);
        let io = match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| io::Error::other("shared HTTP driver terminated unexpectedly"))
                .and_then(std::convert::identity),
            None => Ok(()),
        };
        let processor = match self.processor.take() {
            Some(worker) => worker.finish(),
            None => Ok(()),
        };
        io.and(processor)
    }
}
impl Drop for HttpDriver {
    fn drop(&mut self) {
        if self.join().is_err() {
            tracing::error!("Shared HTTP driver shutdown failed");
        }
    }
}
fn run<B: HttpBackend + 'static, E>(
    factory: impl FnOnce() -> Result<B, E>,
    mut incoming: mpsc::Receiver<Command>,
    stop: &AtomicBool,
) -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(IO_THREADS)
        .max_blocking_threads(MAX_BLOCKING_THREADS)
        .thread_name("tallyline-http-io")
        .enable_all()
        .build()?;
    let backend = {
        let _entered = runtime.enter();
        Arc::new(factory().map_err(|_| io::Error::other("shared HTTP backend is unavailable"))?)
    };
    let mut tasks = JoinSet::new();
    let submissions = Arc::new(tokio::sync::Semaphore::new(MAX_SUBMISSIONS));
    let result = drive(
        &runtime,
        &backend,
        &mut tasks,
        &mut incoming,
        stop,
        &submissions,
    );
    runtime.block_on(tasks.shutdown());
    drop(backend);
    drop(runtime);
    result
}
fn drive<B: HttpBackend + 'static>(
    runtime: &tokio::runtime::Runtime,
    backend: &Arc<B>,
    tasks: &mut JoinSet<()>,
    incoming: &mut mpsc::Receiver<Command>,
    stop: &AtomicBool,
    submissions: &Arc<tokio::sync::Semaphore>,
) -> io::Result<()> {
    while !stop.load(Ordering::Acquire) {
        while let Some(result) = tasks.try_join_next() {
            result.map_err(|_| {
                io::Error::other("shared HTTP request task terminated unexpectedly")
            })?;
        }
        if tasks.len() >= MAX_ACTIVE {
            if let Ok(Some(result)) = runtime
                .block_on(async { tokio::time::timeout(CONTROL_POLL, tasks.join_next()).await })
            {
                result.map_err(|_| {
                    io::Error::other("shared HTTP request task terminated unexpectedly")
                })?;
            }
            continue;
        }
        match runtime.block_on(async { tokio::time::timeout(CONTROL_POLL, incoming.recv()).await })
        {
            Ok(Some(command)) => {
                let backend = Arc::clone(backend);
                tasks.spawn_on(
                    route(backend, command, Arc::clone(submissions))
                        .with_subscriber(tracing::subscriber::NoSubscriber::default()),
                    runtime.handle(),
                );
            }
            Ok(None) => break,
            Err(_) => continue,
        }
    }
    incoming.close();
    Ok(())
}
async fn route<B: HttpBackend + 'static>(
    backend: Arc<B>,
    command: Command,
    submissions: Arc<tokio::sync::Semaphore>,
) {
    match command {
        Command::VerifyRowReceipts(job, values, token, reply) => {
            let result = backend.verify_row_receipts(job, values, token).await;
            if reply.send(result).is_err() {
                tracing::debug!("Row receipt verification reply receiver closed");
            }
        }
        Command::ReserveSubmission(reply) => {
            let Ok(_permit) = submissions.acquire_owned().await else {
                return;
            };
            let (slot, request, result) = crate::api::SubmissionSlot::channel();
            if reply.send(slot).is_err() {
                return;
            }
            let Ok(request) = request.await else {
                return;
            };
            let answer = backend
                .submit(request.prepared, request.values, request.token)
                .await;
            if result.send(answer).is_err() {
                tracing::debug!("Reserved submission HTTP reply receiver closed");
            }
        }
        Command::RecheckRefusal(job, values, token, reply) => {
            let result = backend.recheck_refusal(job, values, token).await;
            if reply.send(result).is_err() {
                tracing::debug!("Refusal recheck reply receiver closed");
            }
        }
        Command::VerifyReceipts(job, values, token, reply) => {
            let result = backend.verify_receipts(job, values, token).await;
            if reply.send(result).is_err() {
                tracing::debug!("Receipt verification reply receiver closed");
            }
        }
        Command::Fetch(endpoint, reply) => {
            let result = backend.fetch(endpoint).await;
            if reply.send(result).is_err() {
                tracing::debug!("Public HTTP reply receiver closed");
            }
        }
        Command::Priors(terminals, reply) => {
            let result = backend.fetch_terminal_priors(terminals).await;
            if reply.send(result).is_err() {
                tracing::debug!("Prior HTTP reply receiver closed");
            }
        }
        Command::Validate(values, reply) => {
            let result = backend.validate(values).await;
            if reply.send(result).is_err() {
                tracing::debug!("Credential HTTP reply receiver closed");
            }
        }
        Command::Submit(prepared, values, token, reply) => {
            let Ok(_permit) = submissions.acquire_owned().await else {
                if reply
                    .send(tallyline_submission::api::model::SubmissionResult::Unknown)
                    .is_err()
                {
                    tracing::debug!("Submission HTTP reply receiver closed");
                }
                return;
            };
            let result = backend.submit(prepared, values, token).await;
            if reply.send(result).is_err() {
                tracing::debug!("Submission HTTP reply receiver closed");
            }
        }
    }
}
