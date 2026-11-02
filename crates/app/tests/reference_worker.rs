// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! App-owned reference-worker contracts through public cache and fetcher ports.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc,
};
use std::time::Duration;
use tallyline_adapter_uex::api::ReadError;
use tallyline_app::api::ReferenceWorker;
use tallyline_reference_data::api::model::{
    CachedReference, Commodity, CommodityAverages, Endpoint, EndpointData, TerminalPriors,
};
use tallyline_reference_data::api::{ReferenceDataCache, ReferenceDataFetcher};
use tallyline_shared_kernel::api::Clock;
use tallyline_shared_kernel::api::model::{CommodityId, TerminalId, Timestamp};
use tallyline_ui::api::model::{BackgroundEvent, ReferenceCommand};

struct MemoryCache {
    entries: Arc<Mutex<Vec<CachedReference>>>,
    writes: Arc<AtomicUsize>,
    read_only: bool,
}
impl ReferenceDataCache for MemoryCache {
    type Error = &'static str;
    fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
        assert!(tokio::runtime::Handle::try_current().is_err());
        self.entries
            .lock()
            .map(|entries| entries.clone())
            .map_err(|_| "cache lock failed")
    }
    fn replace(&mut self, entry: &CachedReference) -> Result<(), Self::Error> {
        assert!(tokio::runtime::Handle::try_current().is_err());
        self.writes.fetch_add(1, Ordering::SeqCst);
        if self.read_only {
            return Err("read only");
        }
        let mut entries = self.entries.lock().map_err(|_| "cache lock failed")?;
        entries.retain(|old| old.endpoint() != entry.endpoint());
        entries.push(entry.clone());
        Ok(())
    }
}
struct TestClock;
impl Clock for TestClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_millis(7200000)
    }
}
struct Cancelled(Arc<AtomicBool>);
impl Drop for Cancelled {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
struct PendingFetcher {
    calls: Arc<AtomicUsize>,
    cancelled: Arc<AtomicBool>,
    entered: mpsc::SyncSender<()>,
}
impl ReferenceDataFetcher for PendingFetcher {
    type Error = ReadError;
    async fn fetch(&self, _: Endpoint) -> Result<EndpointData, ReadError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let guard = Cancelled(Arc::clone(&self.cancelled));
        self.entered
            .try_send(())
            .map_err(|_| ReadError::Transport("test network probe unavailable".to_owned()))?;
        let result = std::future::pending().await;
        drop(guard);
        result
    }
    async fn fetch_terminal_priors(
        &self,
        _: &[TerminalId],
    ) -> Result<Vec<TerminalPriors>, ReadError> {
        Err(ReadError::Transport("unused test batch".to_owned()))
    }
}
fn finish(worker: ReferenceWorker) -> std::io::Result<()> {
    let (completed, result) = mpsc::sync_channel(1);
    let joining = std::thread::spawn(move || completed.send(worker.finish()));
    let outcome = result
        .recv_timeout(Duration::from_secs(5))
        .map_err(std::io::Error::other)?;
    joining
        .join()
        .map_err(|_| std::io::Error::other("test join failed"))?
        .map_err(|_| std::io::Error::other("test completion delivery failed"))?;
    outcome
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_ui_13_r_nf_10__cached_records_publish_while_network_is_pending_and_shutdown_cancels_the_get()
 {
    let entries = Arc::new(Mutex::new(vec![CachedReference::new(
        Timestamp::from_seconds(0),
        EndpointData::Commodities(vec![Commodity::new(
            CommodityId(1),
            "Cached Ore",
            "CACHE",
            true,
            CommodityAverages::default(),
        )]),
    )]));
    let writes = Arc::new(AtomicUsize::new(0));
    let calls = Arc::new(AtomicUsize::new(0));
    let cancelled = Arc::new(AtomicBool::new(false));
    let store = MemoryCache {
        entries,
        writes: Arc::clone(&writes),
        read_only: false,
    };
    let (commands, incoming) = mpsc::sync_channel::<ReferenceCommand>(1);
    let (outgoing, events) = mpsc::sync_channel(16);
    let requested = Arc::clone(&calls);
    let dropped = Arc::clone(&cancelled);
    let (entered, started) = mpsc::sync_channel(1);
    let worker = ReferenceWorker::start(
        store,
        TestClock,
        incoming,
        outgoing,
        move || {
            Ok::<_, ()>(PendingFetcher {
                calls: Arc::clone(&requested),
                cancelled: Arc::clone(&dropped),
                entered: entered.clone(),
            })
        },
        || {},
    )
    .unwrap();
    let BackgroundEvent::ReferenceUpdated {
        live,
        ptu,
        observed_at,
    } = events.recv_timeout(Duration::from_secs(5)).unwrap()
    else {
        panic!("cache must be first")
    };
    assert_eq!(live.commodities("cached").len(), 1);
    assert_eq!(ptu.commodities("cached").len(), 1);
    assert_eq!(observed_at, Timestamp::from_millis(7200000));
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(5)).unwrap(),
        BackgroundEvent::ReferenceRefreshStarted
    ));
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    finish(worker).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(cancelled.load(Ordering::Acquire));
    assert_eq!(writes.load(Ordering::SeqCst), 0);
    drop(commands);
}

fn next_snapshot(
    events: &mpsc::Receiver<BackgroundEvent>,
) -> std::io::Result<(
    tallyline_reference_data::api::model::ReferenceSnapshot,
    tallyline_reference_data::api::model::ReferenceSnapshot,
    Timestamp,
)> {
    loop {
        match events
            .recv_timeout(Duration::from_secs(5))
            .map_err(std::io::Error::other)?
        {
            BackgroundEvent::ReferenceUpdated {
                live,
                ptu,
                observed_at,
            } => return Ok((live, ptu, observed_at)),
            BackgroundEvent::ReferenceRefreshStarted | BackgroundEvent::ReferenceDiagnostic(_) => {}
            other @ (BackgroundEvent::AttachmentPreview { .. }
            | BackgroundEvent::AttachmentStatus { .. }
            | BackgroundEvent::AttachmentFailed { .. }
            | BackgroundEvent::ReportsLoaded { .. }
            | BackgroundEvent::ReportFailed(_)
            | BackgroundEvent::UpdateChecked(_)
            | BackgroundEvent::GameWitnessLoaded(_)
            | BackgroundEvent::Failure(_)
            | BackgroundEvent::SettingsLoaded(_)
            | BackgroundEvent::LicencesLoaded(_)
            | BackgroundEvent::StorageWarning(_)
            | BackgroundEvent::CredentialsUpdated { .. }
            | BackgroundEvent::LicencesUnavailable) => {
                return Err(std::io::Error::other(format!(
                    "unexpected worker event: {other:?}"
                )));
            }
        }
    }
}
struct ReadyFetcher {
    responses: Arc<Vec<(Endpoint, EndpointData)>>,
    calls: Arc<Mutex<Vec<Endpoint>>>,
}
impl ReferenceDataFetcher for ReadyFetcher {
    type Error = ReadError;
    async fn fetch(&self, endpoint: Endpoint) -> Result<EndpointData, ReadError> {
        self.calls
            .lock()
            .map_err(|_| ReadError::Transport("test lock failed".to_owned()))?
            .push(endpoint);
        self.responses
            .iter()
            .find(|(key, _)| *key == endpoint)
            .map(|(_, data)| data.clone())
            .ok_or_else(|| ReadError::Transport("synthetic offline response".to_owned()))
    }
    async fn fetch_terminal_priors(
        &self,
        _: &[TerminalId],
    ) -> Result<Vec<TerminalPriors>, ReadError> {
        Err(ReadError::Transport("unused test batch".to_owned()))
    }
}
fn cached_commodity() -> CachedReference {
    CachedReference::new(
        Timestamp::from_seconds(0),
        EndpointData::Commodities(vec![Commodity::new(
            CommodityId(1),
            "Cached Ore",
            "CACHE",
            true,
            CommodityAverages::default(),
        )]),
    )
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1_r_ui_13__client_initialisation_failure_keeps_the_loaded_cache_usable_offline() {
    use tallyline_reference_data::api::model::ReferenceState;
    let store = MemoryCache {
        entries: Arc::new(Mutex::new(vec![cached_commodity()])),
        writes: Arc::new(AtomicUsize::new(0)),
        read_only: false,
    };
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(16);
    let worker = ReferenceWorker::start(
        store,
        TestClock,
        incoming,
        outgoing,
        || Err::<PendingFetcher, ()>(()),
        || {},
    )
    .unwrap();
    let (cached, _, _) = next_snapshot(&events).unwrap();
    assert_eq!(cached.commodities("cached").len(), 1);
    let (offline, _, _) = next_snapshot(&events).unwrap();
    assert_eq!(offline.state(), ReferenceState::Offline);
    assert_eq!(offline.commodities("cached").len(), 1);
    assert_eq!(
        offline.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(0))
    );
    finish(worker).unwrap();
    drop(commands);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_1__failed_cache_write_never_publishes_new_network_values_or_fetch_time() {
    use tallyline_reference_data::api::model::ReferenceState;
    let original = cached_commodity();
    let entries = Arc::new(Mutex::new(vec![original.clone()]));
    let writes = Arc::new(AtomicUsize::new(0));
    let store = MemoryCache {
        entries: Arc::clone(&entries),
        writes: Arc::clone(&writes),
        read_only: true,
    };
    let responses = Arc::new(vec![(
        Endpoint::Commodities,
        EndpointData::Commodities(vec![Commodity::new(
            CommodityId(2),
            "Network Ore",
            "NET",
            true,
            CommodityAverages::default(),
        )]),
    )]);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(16);
    let worker = ReferenceWorker::start(
        store,
        TestClock,
        incoming,
        outgoing,
        move || {
            Ok::<_, ()>(ReadyFetcher {
                responses: Arc::clone(&responses),
                calls: Arc::clone(&calls),
            })
        },
        || {},
    )
    .unwrap();
    next_snapshot(&events).unwrap();
    let (snapshot, _, _) = next_snapshot(&events).unwrap();
    assert_eq!(snapshot.state(), ReferenceState::Offline);
    assert_eq!(snapshot.commodities("cached").len(), 1);
    assert!(snapshot.commodities("network").is_empty());
    assert_eq!(
        snapshot.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(0))
    );
    finish(worker).unwrap();
    assert_eq!(*entries.lock().unwrap(), vec![original]);
    assert_eq!(writes.load(Ordering::SeqCst), 1);
    drop(commands);
}

struct MovingClock(Arc<std::sync::atomic::AtomicU64>);
impl Clock for MovingClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_millis(self.0.load(Ordering::Acquire))
    }
}
fn fresh_catalogue() -> Vec<CachedReference> {
    use tallyline_reference_data::api::model::{
        AcceptanceSwitch, CommodityConstraints, GameVersions, ReferenceParameters, StatusLevels,
        SubmissionAcceptance, Terminal, TerminalText,
    };
    [
        cached_commodity().data().clone(),
        EndpointData::Terminals(vec![Terminal::new(
            TerminalId(17),
            TerminalText {
                name: "Known Terminal".to_owned(),
                nickname: "Known".to_owned(),
                display_name: "Known".to_owned(),
                location: "Test System".to_owned(),
            },
            true,
        )]),
        EndpointData::GameVersions(GameVersions {
            live: "4.9".to_owned(),
            ptu: None,
        }),
        EndpointData::StatusLevels(StatusLevels {
            buy: vec![],
            sell: vec![],
        }),
        EndpointData::Parameters(ReferenceParameters::new(
            SubmissionAcceptance {
                datacenter: AcceptanceSwitch::Enabled,
                reports: AcceptanceSwitch::Enabled,
                commodity: AcceptanceSwitch::Enabled,
                ptu: AcceptanceSwitch::Enabled,
            },
            CommodityConstraints {
                price_variation_percent: 25,
                prior_ttl_days: std::num::NonZeroU32::MIN,
                scu_variation_percent: 5000,
            },
            None,
        )),
    ]
    .into_iter()
    .map(|data| CachedReference::new(Timestamp::from_seconds(0), data))
    .collect()
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_cap_3b_r_api_2__startup_rechecks_fresh_versions_and_terminal_open_obeys_its_cached_ttl() {
    use tallyline_reference_data::api::model::GameVersions;
    let mut initial = fresh_catalogue();
    initial.push(CachedReference::new(
        Timestamp::from_seconds(0),
        EndpointData::TerminalPriors(TerminalPriors::new(TerminalId(17), vec![]).unwrap()),
    ));
    let calls = Arc::new(Mutex::new(Vec::new()));
    let observed_calls = Arc::clone(&calls);
    let time = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let store = MemoryCache {
        entries: Arc::new(Mutex::new(initial)),
        writes: Arc::new(AtomicUsize::new(0)),
        read_only: false,
    };
    let responses = Arc::new(vec![
        (
            Endpoint::GameVersions,
            EndpointData::GameVersions(GameVersions {
                live: "4.10.2".to_owned(),
                ptu: None,
            }),
        ),
        (
            Endpoint::TerminalPriors(TerminalId(17)),
            EndpointData::TerminalPriors(TerminalPriors::new(TerminalId(17), vec![]).unwrap()),
        ),
    ]);
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(16);
    let worker = ReferenceWorker::start(
        store,
        MovingClock(Arc::clone(&time)),
        incoming,
        outgoing,
        move || {
            Ok::<_, ()>(ReadyFetcher {
                responses: Arc::clone(&responses),
                calls: Arc::clone(&calls),
            })
        },
        || {},
    )
    .unwrap();
    let (cached, _, _) = next_snapshot(&events).unwrap();
    assert_eq!(cached.game_versions().unwrap().live, "4.9");
    let (network, _, _) = next_snapshot(&events).unwrap();
    assert_eq!(network.game_versions().unwrap().live, "4.10.2");
    assert_eq!(
        *observed_calls.lock().unwrap(),
        vec![Endpoint::GameVersions]
    );
    commands
        .send(ReferenceCommand::OpenTerminal(TerminalId(17)))
        .unwrap();
    let (fresh, _, _) = next_snapshot(&events).unwrap();
    assert!(fresh.terminal_priors(TerminalId(17)).is_some());
    assert_eq!(observed_calls.lock().unwrap().len(), 1);
    time.store(1800000, Ordering::Release);
    commands
        .send(ReferenceCommand::OpenTerminal(TerminalId(17)))
        .unwrap();
    let (expired, ptu, _) = next_snapshot(&events).unwrap();
    assert_eq!(
        expired.fetched_at(Endpoint::TerminalPriors(TerminalId(17))),
        Some(Timestamp::from_millis(1800000))
    );
    assert!(ptu.terminal_priors(TerminalId(17)).is_none());
    finish(worker).unwrap();
    assert_eq!(
        *observed_calls.lock().unwrap(),
        vec![
            Endpoint::GameVersions,
            Endpoint::TerminalPriors(TerminalId(17))
        ]
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_nf_10_r_api_1__worker_drop_joins_even_when_the_ui_has_not_drained_its_bounded_events() {
    let store = MemoryCache {
        entries: Arc::new(Mutex::new(vec![cached_commodity()])),
        writes: Arc::new(AtomicUsize::new(0)),
        read_only: false,
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let cancelled = Arc::new(AtomicBool::new(false));
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(1);
    let (entered, _started) = mpsc::sync_channel(1);
    let (awakened, ready) = mpsc::sync_channel(1);
    let worker = ReferenceWorker::start(
        store,
        TestClock,
        incoming,
        outgoing,
        move || {
            Ok::<_, ()>(PendingFetcher {
                calls: Arc::clone(&calls),
                cancelled: Arc::clone(&cancelled),
                entered: entered.clone(),
            })
        },
        move || {
            assert!(awakened.try_send(()).is_ok());
        },
    )
    .unwrap();
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    let (completed, result) = mpsc::sync_channel(1);
    let joining = std::thread::spawn(move || {
        drop(worker);
        completed.send(())
    });
    result.recv_timeout(Duration::from_secs(5)).unwrap();
    joining.join().unwrap().unwrap();
    assert!(matches!(
        events.try_recv().unwrap(),
        BackgroundEvent::ReferenceUpdated { .. }
    ));
    assert_eq!(observed.load(Ordering::SeqCst), 0);
    drop(commands);
}

struct NonJsonFetcher;
impl ReferenceDataFetcher for NonJsonFetcher {
    type Error = ReadError;
    async fn fetch(&self, _: Endpoint) -> Result<EndpointData, ReadError> {
        Err(ReadError::NotJson {
            http_status: 503,
            cf_ray: Some("230b030023ae2822-SJC".to_owned()),
        })
    }
    async fn fetch_terminal_priors(
        &self,
        _: &[TerminalId],
    ) -> Result<Vec<TerminalPriors>, ReadError> {
        Err(ReadError::Transport("unused batch".to_owned()))
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_api_3__non_json_metadata_uses_the_injected_observation_time_without_replacing_cache() {
    use tallyline_ui::api::model::ReferenceDiagnostic;
    let store = MemoryCache {
        entries: Arc::new(Mutex::new(vec![cached_commodity()])),
        writes: Arc::new(AtomicUsize::new(0)),
        read_only: false,
    };
    let (commands, incoming) = mpsc::sync_channel(1);
    let (outgoing, events) = mpsc::sync_channel(16);
    let worker = ReferenceWorker::start(
        store,
        TestClock,
        incoming,
        outgoing,
        || Ok::<_, ()>(NonJsonFetcher),
        || {},
    )
    .unwrap();
    let (cached, _, _) = next_snapshot(&events).unwrap();
    assert_eq!(
        cached.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(0))
    );
    loop {
        match events.recv_timeout(Duration::from_secs(5)).unwrap() {
            BackgroundEvent::ReferenceDiagnostic(diagnostic) => {
                assert_eq!(
                    diagnostic,
                    ReferenceDiagnostic {
                        http_status: 503,
                        cf_ray: Some("230b030023ae2822-SJC".to_owned()),
                        observed_at: Timestamp::from_millis(7200000)
                    }
                );
                break;
            }
            BackgroundEvent::ReferenceRefreshStarted | BackgroundEvent::ReferenceUpdated { .. } => {
            }
            other @ (BackgroundEvent::AttachmentPreview { .. }
            | BackgroundEvent::AttachmentStatus { .. }
            | BackgroundEvent::AttachmentFailed { .. }
            | BackgroundEvent::ReportsLoaded { .. }
            | BackgroundEvent::ReportFailed(_)
            | BackgroundEvent::UpdateChecked(_)
            | BackgroundEvent::GameWitnessLoaded(_)
            | BackgroundEvent::Failure(_)
            | BackgroundEvent::SettingsLoaded(_)
            | BackgroundEvent::LicencesLoaded(_)
            | BackgroundEvent::StorageWarning(_)
            | BackgroundEvent::CredentialsUpdated { .. }
            | BackgroundEvent::LicencesUnavailable) => panic!("unexpected event: {other:?}"),
        }
    }
    let (offline, _, _) = next_snapshot(&events).unwrap();
    assert_eq!(offline.commodities("cached").len(), 1);
    assert_eq!(
        offline.fetched_at(Endpoint::Commodities),
        Some(Timestamp::from_seconds(0))
    );
    finish(worker).unwrap();
    drop(commands);
}
