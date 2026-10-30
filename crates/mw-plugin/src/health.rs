//! Process-wide counters of failed plugin calls for the admin Health panel and
//! `/metrics` (t103). Every guest-export failure passes through
//! [`crate::host_state::map_call_err`], which records it here by kind.
//!
//! Bounded: timestamps of the last hour only, at most [`MAX_EVENTS`] per kind.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(3600);
const MAX_EVENTS: usize = 4096;

/// Why a guest call failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallFailure {
    /// A deadline, memory or fuel limit tripped.
    Limit,
    /// The guest trapped (unreachable, out-of-bounds, ...).
    Trap,
    /// Any other failure at the call boundary.
    Runtime,
}

struct Counters {
    totals: [AtomicU64; 3],
    recent: Mutex<[VecDeque<Instant>; 3]>,
}

static COUNTERS: Counters = Counters {
    totals: [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)],
    recent: Mutex::new([VecDeque::new(), VecDeque::new(), VecDeque::new()]),
};

fn index(kind: CallFailure) -> usize {
    match kind {
        CallFailure::Limit => 0,
        CallFailure::Trap => 1,
        CallFailure::Runtime => 2,
    }
}

fn prune(ring: &mut VecDeque<Instant>, now: Instant) {
    while ring
        .front()
        .is_some_and(|t| now.duration_since(*t) > WINDOW)
        || ring.len() > MAX_EVENTS
    {
        ring.pop_front();
    }
}

pub(crate) fn record(kind: CallFailure) {
    let i = index(kind);
    COUNTERS.totals[i].fetch_add(1, Ordering::Relaxed);
    let now = Instant::now();
    let mut recent = COUNTERS.recent.lock().unwrap_or_else(|e| e.into_inner());
    recent[i].push_back(now);
    prune(&mut recent[i], now);
}

/// Failed guest calls of every kind since this process started, and in the
/// last hour.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CallFailures {
    pub limit_total: u64,
    pub trap_total: u64,
    pub runtime_total: u64,
    pub limit_last_hour: u64,
    pub trap_last_hour: u64,
    pub runtime_last_hour: u64,
}

pub fn call_failures() -> CallFailures {
    let now = Instant::now();
    let mut recent = COUNTERS.recent.lock().unwrap_or_else(|e| e.into_inner());
    for ring in recent.iter_mut() {
        prune(ring, now);
    }
    let total = |k| COUNTERS.totals[index(k)].load(Ordering::Relaxed);
    CallFailures {
        limit_total: total(CallFailure::Limit),
        trap_total: total(CallFailure::Trap),
        runtime_total: total(CallFailure::Runtime),
        limit_last_hour: recent[0].len() as u64,
        trap_last_hour: recent[1].len() as u64,
        runtime_last_hour: recent[2].len() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failures_are_counted_by_kind() {
        let before = call_failures();
        record(CallFailure::Trap);
        record(CallFailure::Limit);
        let after = call_failures();
        assert!(after.trap_total > before.trap_total);
        assert!(after.limit_total > before.limit_total);
        assert!(after.trap_last_hour >= 1 && after.limit_last_hour >= 1);
    }
}
