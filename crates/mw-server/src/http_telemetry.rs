//! Optional response telemetry has a bounded, dedicated aggregation thread.
//! Request tasks never wait for aggregation, persistence or queue capacity.
use crate::{http_metrics::RouteClass, request_history};
use serde::Serialize;
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

const CAPACITY: usize = 8192;
const BATCH: usize = 256;
static WORKER: OnceLock<Arc<Worker>> = OnceLock::new();

#[derive(Default)]
struct Stats {
    submitted: AtomicU64,
    applied: AtomicU64,
    dropped: AtomicU64,
    pending: AtomicU64,
    running: AtomicBool,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    pub submitted: u64,
    pub applied: u64,
    pub dropped: u64,
    /// Accepted/admitting samples not yet applied or dropped, including work in progress.
    pub depth: u64,
    pub queue_depth: usize,
    pub capacity: usize,
    pub running: bool,
    pub closing: bool,
}

struct Sample {
    history: Arc<request_history::Recorder>,
    class: RouteClass,
    status: u16,
    micros: u64,
    finished_at: i64,
    // Only the existing bounded redactor's output is retained, never a URI.
    missing: Option<(&'static str, String)>,
    stats: Arc<Stats>,
    applied: bool,
}
impl Sample {
    fn apply(mut self) {
        crate::http_metrics::record_us_at(
            self.finished_at.max(0) as u64,
            self.class,
            self.status,
            self.micros,
        );
        self.history.record_at(
            self.finished_at,
            self.status,
            self.micros,
            self.missing
                .as_ref()
                .map(|(method, path)| (*method, path.as_str())),
        );
        self.applied = true;
        self.stats.applied.fetch_add(1, Ordering::Relaxed);
    }
}
impl Drop for Sample {
    fn drop(&mut self) {
        if !self.applied {
            self.stats.dropped.fetch_add(1, Ordering::Relaxed);
        }
        self.stats.pending.fetch_sub(1, Ordering::Relaxed);
    }
}
enum Message {
    Sample(Sample),
    Barrier(oneshot::Sender<()>),
    Stop(oneshot::Sender<()>),
}

pub(crate) struct Worker {
    sender: mpsc::Sender<Message>,
    stats: Arc<Stats>,
    closing: AtomicBool,
}

impl Worker {
    fn new(capacity: usize, before_receive: impl FnOnce() + Send + 'static) -> Arc<Self> {
        let (sender, mut receiver) = mpsc::channel(capacity);
        let stats = Arc::new(Stats::default());
        let worker = Arc::new(Self {
            sender,
            stats: stats.clone(),
            closing: AtomicBool::new(false),
        });
        stats.running.store(true, Ordering::Release);
        let stopped = stats.clone();
        let result = std::thread::Builder::new()
            .name("mw-http-telemetry".into())
            .spawn(move || {
                struct Running(Arc<Stats>);
                impl Drop for Running {
                    fn drop(&mut self) {
                        self.0.running.store(false, Ordering::Release);
                    }
                }
                let _running = Running(stopped);
                before_receive();
                let mut stop = None;
                while let Some(message) = receiver.blocking_recv() {
                    process(message, &mut receiver, &mut stop);
                    for _ in 1..BATCH {
                        let Ok(message) = receiver.try_recv() else {
                            break;
                        };
                        process(message, &mut receiver, &mut stop);
                    }
                }
                // Closing the receiver still drains messages accepted just before Stop.
                drop(_running);
                if let Some(ack) = stop {
                    let _ = ack.send(());
                }
            });
        if let Err(error) = result {
            stats.running.store(false, Ordering::Release);
            tracing::warn!("HTTP telemetry worker unavailable: {error}");
        }
        worker
    }

    pub(crate) fn submit(
        &self,
        history: &Arc<request_history::Recorder>,
        class: RouteClass,
        status: u16,
        micros: u64,
        method: &axum::http::Method,
        path: &str,
    ) {
        let finished_at = chrono::Utc::now().timestamp();
        self.submit_at(
            history,
            class,
            status,
            micros,
            finished_at,
            request_history::redacted_not_found(status, method, path),
        );
    }
    fn submit_at(
        &self,
        history: &Arc<request_history::Recorder>,
        class: RouteClass,
        status: u16,
        micros: u64,
        finished_at: i64,
        missing: Option<(&'static str, String)>,
    ) {
        self.stats.submitted.fetch_add(1, Ordering::Relaxed);
        self.stats.pending.fetch_add(1, Ordering::Relaxed);
        let sample = Sample {
            history: history.clone(),
            class,
            status,
            micros,
            finished_at,
            missing,
            stats: self.stats.clone(),
            applied: false,
        };
        if self.closing.load(Ordering::Acquire) {
            return;
        }
        // A failed send drops its Sample, accounting for full/disconnected queues.
        let _ = self.sender.try_send(Message::Sample(sample));
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            submitted: self.stats.submitted.load(Ordering::Relaxed),
            applied: self.stats.applied.load(Ordering::Relaxed),
            dropped: self.stats.dropped.load(Ordering::Relaxed),
            depth: self.stats.pending.load(Ordering::Relaxed),
            queue_depth: self.sender.max_capacity() - self.sender.capacity(),
            capacity: self.sender.max_capacity(),
            running: self.stats.running.load(Ordering::Acquire),
            closing: self.closing.load(Ordering::Acquire),
        }
    }
    async fn barrier(&self, stop: bool, timeout: Duration) -> bool {
        if stop
            && self.closing.load(Ordering::Acquire)
            && !self.stats.running.load(Ordering::Acquire)
            && self.stats.pending.load(Ordering::Relaxed) == 0
        {
            return true;
        }
        if stop {
            self.closing.store(true, Ordering::Release);
        }
        let (sender, receiver) = oneshot::channel();
        tokio::time::timeout(timeout, async {
            self.sender
                .send(if stop {
                    Message::Stop(sender)
                } else {
                    Message::Barrier(sender)
                })
                .await
                .ok()?;
            receiver.await.ok()
        })
        .await
        .is_ok_and(|result| result.is_some())
    }
}
fn process(
    message: Message,
    receiver: &mut mpsc::Receiver<Message>,
    stop: &mut Option<oneshot::Sender<()>>,
) {
    match message {
        Message::Sample(sample) => sample.apply(),
        Message::Barrier(ack) => {
            let _ = ack.send(());
        }
        Message::Stop(ack) => {
            receiver.close();
            *stop = Some(ack);
        }
    }
}
pub(crate) fn start() -> Arc<Worker> {
    WORKER.get_or_init(|| Worker::new(CAPACITY, || {})).clone()
}
pub(crate) fn snapshot() -> Snapshot {
    WORKER.get().map_or(
        Snapshot {
            capacity: CAPACITY,
            ..Default::default()
        },
        |worker| worker.snapshot(),
    )
}
pub(crate) async fn flush() -> bool {
    match WORKER.get() {
        Some(worker) => worker.barrier(false, Duration::from_secs(2)).await,
        None => true,
    }
}
/// Stop accepting new samples, drain already accepted observations, bounded to two seconds.
pub async fn shutdown() -> bool {
    match WORKER.get() {
        Some(worker) => worker.barrier(true, Duration::from_secs(2)).await,
        None => true,
    }
}

#[cfg(test)]
#[path = "http_telemetry_tests.rs"]
mod tests;
