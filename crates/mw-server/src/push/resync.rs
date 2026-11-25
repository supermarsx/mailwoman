//! Reading an account's current state after a gap (t35).
//!
//! A stream that missed broadcast changes (its receiver lagged, or the engine
//! bridge lagged and every stream was asked to resync) reads the current state
//! of each account it carries and sends that. State tokens are absolute, so the
//! value read covers every skipped change, provided the read **started after**
//! the gap was detected: the skipped changes were committed before they were
//! broadcast, so before the gap was seen.
//!
//! When many streams resync at once (a bridge lag resyncs every stream), reads
//! of one account are shared through [`Hub`]: a requester joins a read already
//! in flight only when that read had not started, or started after the
//! requester's gap. Otherwise it starts its own. Reads to the store are bounded
//! by a semaphore (`MW_REALTIME_RESYNC_CONCURRENCY`).
//!
//! "Started" and "detected" are positions in one sequence ([`Hub::gap`] and the
//! start of a read both take the next number), not clock readings, so the rule
//! does not depend on clock resolution.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::FutureExt;
use futures_util::future::{BoxFuture, Shared};
use tokio::sync::Semaphore;

use mw_engine::StateChange;

use super::metrics;

/// Reads one account's current state. The error is a short reason for the
/// log; it never carries an account id.
pub(crate) type ReadFn =
    Arc<dyn Fn(String) -> BoxFuture<'static, Result<StateChange, String>> + Send + Sync>;

/// The store-backed [`ReadFn`] production streams use.
pub(crate) fn store_reader(store: mw_store::Store) -> ReadFn {
    Arc::new(move |account| {
        let store = store.clone();
        Box::pin(async move {
            mw_engine::change::state_change_from_store(&store, &account)
                .await
                .map_err(|_| "the store could not be read".to_string())
        })
    })
}

/// The position at which a requester detected its gap. Only reads that start
/// after it may answer the requester.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Gap(u64);

type Outcome = Result<StateChange, String>;

#[cfg(test)]
pub(crate) type StartHook = Arc<dyn Fn(&str) + Send + Sync>;

struct Flight {
    /// The sequence position at which the read started; 0 while it waits for a
    /// permit.
    started: Arc<AtomicU64>,
    result: Shared<BoxFuture<'static, Outcome>>,
}

impl Flight {
    fn answers(&self, gap: Gap) -> bool {
        let started = self.started.load(SeqCst);
        started == 0 || started > gap.0
    }
}

/// Shared, bounded resync reads for one [`super::PushHandle`].
pub(crate) struct Hub {
    seq: Arc<AtomicU64>,
    permits: Arc<Semaphore>,
    inflight: Mutex<HashMap<String, Arc<Flight>>>,
    /// Test hook: called with the account once a read has its start position,
    /// before the store is read.
    #[cfg(test)]
    pub(crate) on_start: Mutex<Option<StartHook>>,
}

impl Hub {
    pub(crate) fn new(concurrency: usize) -> Self {
        Self {
            seq: Arc::new(AtomicU64::new(0)),
            permits: Arc::new(Semaphore::new(concurrency.max(1))),
            inflight: Mutex::new(HashMap::new()),
            #[cfg(test)]
            on_start: Mutex::new(None),
        }
    }

    /// Whether no read is left in the shared map.
    #[cfg(test)]
    pub(crate) fn idle(&self) -> bool {
        self.inflight.lock().unwrap().is_empty()
    }

    /// Record that a gap was detected now. Call after observing the gap and
    /// before reading.
    pub(crate) fn gap(&self) -> Gap {
        Gap(self.seq.fetch_add(1, SeqCst) + 1)
    }

    /// `account`'s current state, read by a read that started after `gap`.
    /// Gives up after `deadline`.
    pub(crate) async fn read(
        &self,
        source: &ReadFn,
        account: &str,
        gap: Gap,
        deadline: Duration,
    ) -> Outcome {
        let flight = {
            let mut inflight = self.inflight.lock().unwrap_or_else(|e| e.into_inner());
            match inflight.get(account) {
                Some(flight) if flight.answers(gap) => {
                    metrics::resync_read(true);
                    flight.clone()
                }
                _ => {
                    metrics::resync_read(false);
                    let flight = Arc::new(self.start(source, account));
                    inflight.insert(account.to_string(), flight.clone());
                    flight
                }
            }
        };
        let outcome = tokio::time::timeout(deadline, flight.result.clone()).await;
        let timed_out = outcome.is_err();
        let mut inflight = self.inflight.lock().unwrap_or_else(|e| e.into_inner());
        // A finished read leaves the map (a later gap needs a later read); so
        // does one this requester gave up on, so the map never keeps a read
        // nobody is waiting for.
        if inflight
            .get(account)
            .is_some_and(|current| Arc::ptr_eq(current, &flight))
            && (timed_out || flight.result.peek().is_some())
        {
            inflight.remove(account);
        }
        outcome.unwrap_or_else(|_| Err("the state read timed out".into()))
    }

    fn start(&self, source: &ReadFn, account: &str) -> Flight {
        let started = Arc::new(AtomicU64::new(0));
        let (seq, permits, source) = (self.seq.clone(), self.permits.clone(), source.clone());
        let (mark, account) = (started.clone(), account.to_string());
        #[cfg(test)]
        let hook = self.on_start.lock().unwrap().clone();
        let result = async move {
            let _permit = permits
                .acquire_owned()
                .await
                .map_err(|_| "resync reads are closed".to_string())?;
            mark.store(seq.fetch_add(1, SeqCst) + 1, SeqCst);
            #[cfg(test)]
            if let Some(hook) = hook {
                hook(&account);
            }
            let read = source(account.clone()).await?;
            if read.account_id != account {
                return Err("the state read named another account".into());
            }
            Ok(read)
        };
        // A panic inside the read would poison the Shared: every requester
        // joined to it would panic too, and the cleanup in `read` would not
        // run. Contained here (t55): it fails this read's requesters only, as
        // an error, and the finished flight leaves the map like any other.
        let result = std::panic::AssertUnwindSafe(result)
            .catch_unwind()
            .map(|outcome| {
                outcome.unwrap_or_else(|_| {
                    metrics::resync_read_panic();
                    tracing::error!("a realtime state read panicked; its requesters get an error");
                    Err("the state read failed".into())
                })
            })
            .boxed()
            .shared();
        Flight { started, result }
    }
}
