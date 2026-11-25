//! Realtime stream counters (t35).
//!
//! Each event is counted twice: in a process-wide atomic, read by
//! [`snapshot`] (the load fixture's diagnostics snapshot and tests), and through
//! the `metrics` facade (`mw_realtime_*`, the same facade `http_metrics.rs`
//! uses), so an installed exporter sees it too. No label or field carries an
//! account id, a session id or a link id.

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering::Relaxed};

/// Which transport a stream uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Transport {
    Ws,
    Sse,
}

impl Transport {
    fn as_str(self) -> &'static str {
        match self {
            Transport::Ws => "ws",
            Transport::Sse => "sse",
        }
    }
    fn index(self) -> usize {
        self as usize
    }
}

/// Which receiver of a broadcast fell behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Consumer {
    Ws,
    Sse,
    /// The engine → server bridge ([`super::bridge_engine`]).
    Bridge,
}

impl Consumer {
    fn as_str(self) -> &'static str {
        match self {
            Consumer::Ws => "ws",
            Consumer::Sse => "sse",
            Consumer::Bridge => "bridge",
        }
    }
    fn index(self) -> usize {
        self as usize
    }
}

impl From<Transport> for Consumer {
    fn from(t: Transport) -> Self {
        match t {
            Transport::Ws => Consumer::Ws,
            Transport::Sse => Consumer::Sse,
        }
    }
}

/// Why a stream read its accounts' current state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResyncReason {
    /// The stream's own receiver lagged.
    Lag,
    /// [`super::PushHandle::resync_all`] after the engine bridge lagged.
    Bridge,
    /// [`super::PushHandle::resync_all`] from a cross-process transport gap.
    Transport,
}

impl ResyncReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ResyncReason::Lag => "lag",
            ResyncReason::Bridge => "bridge",
            ResyncReason::Transport => "transport",
        }
    }
    fn index(self) -> usize {
        self as usize
    }
}

static STREAMS: [AtomicI64; 2] = [AtomicI64::new(0), AtomicI64::new(0)];
static LAGGED: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static LAGGED_EVENTS: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static RESYNCS: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static RESYNC_SIGNALS: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static RESYNC_FAILURES: AtomicU64 = AtomicU64::new(0);
static RESYNC_READS: AtomicU64 = AtomicU64::new(0);
static RESYNC_READS_SHARED: AtomicU64 = AtomicU64::new(0);
static DUPLICATES_SUPPRESSED: AtomicU64 = AtomicU64::new(0);
static SHUTDOWN_CLOSES: AtomicU64 = AtomicU64::new(0);
static BACKLOG_MAX: AtomicU64 = AtomicU64::new(0);
static PENDING_MAX: AtomicU64 = AtomicU64::new(0);

/// Counts one open stream for as long as it lives.
pub(crate) struct StreamGauge(Transport);

impl StreamGauge {
    pub(crate) fn open(transport: Transport) -> Self {
        STREAMS[transport.index()].fetch_add(1, Relaxed);
        metrics::gauge!("mw_realtime_streams", "transport" => transport.as_str()).increment(1.0);
        Self(transport)
    }
}

impl Drop for StreamGauge {
    fn drop(&mut self) {
        STREAMS[self.0.index()].fetch_sub(1, Relaxed);
        metrics::gauge!("mw_realtime_streams", "transport" => self.0.as_str()).decrement(1.0);
    }
}

/// A receiver reported `Lagged(skipped)`.
pub(crate) fn lagged(consumer: Consumer, skipped: u64) {
    LAGGED[consumer.index()].fetch_add(1, Relaxed);
    LAGGED_EVENTS[consumer.index()].fetch_add(skipped, Relaxed);
    metrics::counter!("mw_realtime_lagged_total", "consumer" => consumer.as_str()).increment(1);
    metrics::counter!("mw_realtime_lagged_events_total", "consumer" => consumer.as_str())
        .increment(skipped);
}

/// One stream started a resync.
pub(crate) fn resync(reason: ResyncReason) {
    RESYNCS[reason.index()].fetch_add(1, Relaxed);
    metrics::counter!("mw_realtime_resyncs_total", "reason" => reason.as_str()).increment(1);
}

/// [`super::PushHandle::resync_all`] was called.
pub(crate) fn resync_signal(reason: ResyncReason) {
    RESYNC_SIGNALS[reason.index()].fetch_add(1, Relaxed);
    metrics::counter!("mw_realtime_resync_signals_total", "reason" => reason.as_str()).increment(1);
}

/// A resync could not read the state it needed; the stream was closed.
pub(crate) fn resync_failure() {
    RESYNC_FAILURES.fetch_add(1, Relaxed);
    metrics::counter!("mw_realtime_resync_failures_total").increment(1);
}

/// A resync read panicked and was contained (t55).
pub(crate) fn resync_read_panic() {
    metrics::counter!("mw_realtime_resync_read_panics_total").increment(1);
}

/// A resync read went to the store (`shared == false`) or joined a read that
/// started after the requester's gap (`shared == true`).
pub(crate) fn resync_read(shared: bool) {
    if shared {
        RESYNC_READS_SHARED.fetch_add(1, Relaxed);
        metrics::counter!("mw_realtime_resync_reads_shared_total").increment(1);
    } else {
        RESYNC_READS.fetch_add(1, Relaxed);
        metrics::counter!("mw_realtime_resync_reads_total").increment(1);
    }
}

/// A frame identical to the last one this stream sent for the same account or
/// link was not sent again.
pub(crate) fn duplicate_suppressed() {
    DUPLICATES_SUPPRESSED.fetch_add(1, Relaxed);
    metrics::counter!("mw_realtime_duplicates_suppressed_total").increment(1);
}

/// A stream was closed because the process is shutting down.
pub(crate) fn shutdown_close() {
    SHUTDOWN_CLOSES.fetch_add(1, Relaxed);
    metrics::counter!("mw_realtime_shutdown_closes_total").increment(1);
}

/// How many broadcast changes were still queued for a stream's receiver when
/// it took one (its backlog); the snapshot keeps the largest seen.
pub(crate) fn observe_backlog(queued: usize) {
    BACKLOG_MAX.fetch_max(queued as u64, Relaxed);
}

/// How many frames a stream had queued to write; the snapshot keeps the
/// largest seen.
pub(crate) fn observe_pending(queued: usize) {
    PENDING_MAX.fetch_max(queued as u64, Relaxed);
}

fn by<const N: usize, T: Copy>(
    cells: &[AtomicU64; N],
    keys: [T; N],
    name: impl Fn(T) -> &'static str,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (cell, key) in cells.iter().zip(keys) {
        map.insert(name(key).into(), cell.load(Relaxed).into());
    }
    serde_json::Value::Object(map)
}

/// The counters since process start, as JSON (no identities).
pub fn snapshot() -> serde_json::Value {
    let consumers = [Consumer::Ws, Consumer::Sse, Consumer::Bridge];
    let reasons = [
        ResyncReason::Lag,
        ResyncReason::Bridge,
        ResyncReason::Transport,
    ];
    serde_json::json!({
        "streams": {
            "ws": STREAMS[Transport::Ws.index()].load(Relaxed),
            "sse": STREAMS[Transport::Sse.index()].load(Relaxed),
        },
        "lagged": by(&LAGGED, consumers, Consumer::as_str),
        "laggedEvents": by(&LAGGED_EVENTS, consumers, Consumer::as_str),
        "resyncs": by(&RESYNCS, reasons, ResyncReason::as_str),
        "resyncSignals": by(&RESYNC_SIGNALS, reasons, ResyncReason::as_str),
        "resyncFailures": RESYNC_FAILURES.load(Relaxed),
        "resyncReads": RESYNC_READS.load(Relaxed),
        "resyncReadsShared": RESYNC_READS_SHARED.load(Relaxed),
        "duplicatesSuppressed": DUPLICATES_SUPPRESSED.load(Relaxed),
        "shutdownCloses": SHUTDOWN_CLOSES.load(Relaxed),
        "streamBacklogMax": BACKLOG_MAX.load(Relaxed),
        "streamPendingMax": PENDING_MAX.load(Relaxed),
    })
}
