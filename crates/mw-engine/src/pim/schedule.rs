//! Shared manual/background synchronization admission and failure backoff.
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
#[derive(Default)]
struct History {
    finished: Option<Instant>,
    failures: u8,
}
#[derive(Default)]
pub(crate) struct SyncControl {
    gate: Arc<tokio::sync::Mutex<()>>,
    history: Mutex<History>,
}
impl SyncControl {
    pub(crate) fn due(&self, interval: Duration) -> bool {
        if interval.is_zero() || self.gate.try_lock().is_err() {
            return false;
        }
        let history = self.history.lock().expect("PIM history");
        let delay = interval
            .saturating_mul(1u32 << history.failures.min(6))
            .min(Duration::from_secs(86400));
        history.finished.is_none_or(|at| at.elapsed() >= delay)
    }
    pub(crate) async fn begin(self: &Arc<Self>) -> SyncRun {
        let guard = self.gate.clone().lock_owned().await;
        SyncRun {
            control: self.clone(),
            _guard: guard,
            success: false,
        }
    }
}
pub(crate) struct SyncRun {
    control: Arc<SyncControl>,
    _guard: tokio::sync::OwnedMutexGuard<()>,
    success: bool,
}
impl SyncRun {
    pub(crate) fn succeeded(&mut self) {
        self.success = true;
    }
}
impl Drop for SyncRun {
    fn drop(&mut self) {
        let mut h = self.control.history.lock().expect("PIM history");
        h.finished = Some(Instant::now());
        h.failures = if self.success {
            0
        } else {
            h.failures.saturating_add(1)
        };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn manual_and_auto_share_one_gate_and_cancel_backs_off() {
        let control = Arc::new(SyncControl::default());
        assert!(control.due(Duration::from_secs(300)));
        assert!(!control.due(Duration::ZERO));
        let first = control.begin().await;
        assert!(!control.due(Duration::from_secs(300)));
        assert!(
            tokio::time::timeout(Duration::from_millis(10), control.begin())
                .await
                .is_err()
        );
        drop(first);
        assert!(!control.due(Duration::from_secs(300)));
        let mut second = control.begin().await;
        second.succeeded();
        drop(second);
        assert_eq!(control.history.lock().unwrap().failures, 0);
    }
    #[test]
    fn failures_back_off_and_success_interval_is_respected() {
        let c = SyncControl::default();
        *c.history.lock().unwrap() = History {
            finished: Some(Instant::now() - Duration::from_secs(450)),
            failures: 1,
        };
        assert!(!c.due(Duration::from_secs(300)));
        c.history.lock().unwrap().failures = 0;
        assert!(c.due(Duration::from_secs(300)));
    }
}
