// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded serial credential work outside the UI thread.

use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError},
};
use std::thread::JoinHandle;
use std::time::Duration;
use tallyline_shared_kernel::api::Clock;
use tallyline_submission::api::model::{
    CredentialFailure, Persistence, ReleaseToken, SavedCredentials, StorageFailure,
};
use tallyline_submission::api::{CredentialApplication, CredentialValidator, SecretStore};
use tallyline_ui::api::model::{BackgroundEvent, CredentialCommand};

/// Store instances created on the credential worker without reading the fallback record.
pub struct CredentialStores<S> {
    primary: Result<S, StorageFailure>,
    fallback: Option<S>,
    token: Option<ReleaseToken>,
}
impl<S> CredentialStores<S> {
    /// Supplies packaged token data prepared on this same worker.
    #[must_use]
    pub fn with_release_token(mut self, token: Option<ReleaseToken>) -> Self {
        self.token = token;
        self
    }
    /// Supplies an OS store and an optional independently checked Linux fallback.
    #[must_use]
    pub const fn new(primary: Result<S, StorageFailure>, fallback: Option<S>) -> Self {
        Self {
            primary,
            fallback,
            token: None,
        }
    }
}
impl<S> From<Result<S, StorageFailure>> for CredentialStores<S> {
    fn from(primary: Result<S, StorageFailure>) -> Self {
        Self::new(primary, None)
    }
}

/// Owns one serial credential runtime and joins before the writer lease ends.
pub struct CredentialWorker {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<io::Result<()>>>,
}
impl CredentialWorker {
    /// Starts isolated credential work without inheriting dependency logging subscribers.
    ///
    /// # Errors
    /// Returns a thread creation failure.
    pub fn start<S, V, C, F, Fut, I>(
        initialise: F,
        validator: V,
        clock: C,
        token: Option<ReleaseToken>,
        incoming: Receiver<CredentialCommand>,
        outgoing: SyncSender<BackgroundEvent>,
        wake: impl Fn() + Send + 'static,
    ) -> io::Result<Self>
    where
        S: SecretStore + 'static,
        V: CredentialValidator + Send + 'static,
        C: Clock + Send + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = I>,
        I: Into<CredentialStores<S>>,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = Arc::clone(&stop);
        let worker = crate::executor::spawn("tallyline-credentials", move || {
            tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                let stores = runtime.block_on(initialise()).into();
                let mut session = Session {
                    application: CredentialApplication::default(),
                    store: stores.primary,
                    fallback: stores.fallback,
                    validator,
                    clock,
                    token: stores.token.or(token),
                    reconcile: false,
                    fallback_available: false,
                };
                let failure = runtime.block_on(session.restore());
                session.fallback_available = failure
                    == Some(CredentialFailure::Storage(StorageFailure::Unavailable))
                    && session.fallback.is_some();
                deliver(&outgoing, session.event(failure), &wake, &stopping)?;
                while !stopping.load(Ordering::Acquire) {
                    let command = match incoming.recv_timeout(Duration::from_millis(100)) {
                        Ok(command) => command,
                        Err(RecvTimeoutError::Timeout) => continue,
                        Err(RecvTimeoutError::Disconnected) => break,
                    };
                    let failure = runtime.block_on(session.process(command));
                    session.reconcile |=
                        failure == Some(CredentialFailure::Storage(StorageFailure::OutcomeUnknown));
                    deliver(&outgoing, session.event(failure), &wake, &stopping)?;
                }
                Ok(())
            })
        })?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
    /// Stops accepting commands and joins the bounded worker.
    ///
    /// # Errors
    /// Returns a runtime failure or worker panic.
    pub fn finish(mut self) -> io::Result<()> {
        self.join()
    }
    fn join(&mut self) -> io::Result<()> {
        self.stop.store(true, Ordering::Release);
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| io::Error::other("credential worker terminated unexpectedly"))?,
            None => Ok(()),
        }
    }
}
impl Drop for CredentialWorker {
    fn drop(&mut self) {
        if self.join().is_err() {
            tracing::error!("Credential worker shutdown failed");
        }
    }
}
struct Session<S, V, C> {
    application: CredentialApplication,
    store: Result<S, StorageFailure>,
    fallback: Option<S>,
    validator: V,
    clock: C,
    token: Option<ReleaseToken>,
    reconcile: bool,
    fallback_available: bool,
}
impl<S: SecretStore, V: CredentialValidator, C: Clock> Session<S, V, C> {
    async fn restore(&mut self) -> Option<CredentialFailure> {
        let store = match &mut self.store {
            Ok(store) => store,
            Err(failure) => return Some(CredentialFailure::Storage(*failure)),
        };
        let restored = match CredentialApplication::load(store).await {
            Ok(application) => application,
            Err(failure) => return Some(CredentialFailure::Storage(failure)),
        };
        if matches!(
            restored.state(),
            tallyline_submission::api::model::AccountState::Unconfigured
        ) {
            return None;
        }
        self.application = restored;
        let request = self.application.begin_revalidation()?;
        let result = request
            .validate_with_release_token(
                &self.validator,
                self.token.as_ref().map(ReleaseToken::for_validation),
            )
            .await;
        let failure = self
            .application
            .accept(request, result, self.clock.now(), store)
            .await
            .err();
        self.reconcile |=
            failure == Some(CredentialFailure::Storage(StorageFailure::OutcomeUnknown));
        failure
    }
    async fn process(&mut self, command: CredentialCommand) -> Option<CredentialFailure> {
        let candidate = match command {
            CredentialCommand::ConfirmFallback => {
                if !self.fallback_available || self.reconcile {
                    return Some(CredentialFailure::Storage(StorageFailure::Unavailable));
                }
                let fallback = self.fallback.take()?;
                self.store = Ok(fallback);
                self.fallback_available = false;
                return self.restore().await;
            }
            CredentialCommand::Validate(candidate) => candidate,
        };
        if self.reconcile && candidate.persistence() == Persistence::Durable {
            let loaded = match &mut self.store {
                Ok(store) => store.load().await.map(|_| ()),
                Err(failure) => Err(*failure),
            };
            if let Err(failure) = loaded {
                return Some(CredentialFailure::Storage(failure));
            }
            self.reconcile = false;
        }
        let request = self.application.begin(candidate);
        let result = request
            .validate_with_release_token(
                &self.validator,
                self.token.as_ref().map(ReleaseToken::for_validation),
            )
            .await;
        match &mut self.store {
            Ok(store) => self
                .application
                .accept(request, result, self.clock.now(), store)
                .await
                .err(),
            Err(failure) => self
                .application
                .accept(
                    request,
                    result,
                    self.clock.now(),
                    &mut UnavailableStore(*failure),
                )
                .await
                .err(),
        }
    }
    fn event(&self, failure: Option<CredentialFailure>) -> BackgroundEvent {
        BackgroundEvent::CredentialsUpdated {
            account: self.application.state().clone(),
            failure,
            available_token: self.token.is_some(),
            fallback_available: self.fallback_available,
        }
    }
}
struct UnavailableStore(StorageFailure);
impl SecretStore for UnavailableStore {
    async fn load(&mut self) -> Result<Option<SavedCredentials>, StorageFailure> {
        Err(self.0)
    }
    async fn save(&mut self, _: &SavedCredentials) -> Result<(), StorageFailure> {
        Err(self.0)
    }
    async fn save_passive(&mut self, _: &SavedCredentials) -> Result<(), StorageFailure> {
        Err(self.0)
    }
    async fn remove(&mut self) -> Result<(), StorageFailure> {
        Err(self.0)
    }
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
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(TrySendError::Disconnected(_)) => return Ok(()),
        }
    }
}
