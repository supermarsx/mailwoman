//! In-process counters per class and action, plus a throttle for audit rows.
//! Counters reset at restart; the durable record is the throttled audit event.
use super::config::{Class, Config, Mode};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{
        Mutex,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

struct Counter {
    detected: AtomicU64,
    enforced: AtomicU64,
    last_seen: AtomicI64,
}
static COUNTERS: [Counter; 11] = [const {
    Counter {
        detected: AtomicU64::new(0),
        enforced: AtomicU64::new(0),
        last_seen: AtomicI64::new(0),
    }
}; 11];

pub(crate) fn record(class: Class, action: Mode) {
    let counter = &COUNTERS[class.index()];
    match action {
        Mode::Enforce => counter.enforced.fetch_add(1, Ordering::Relaxed),
        _ => counter.detected.fetch_add(1, Ordering::Relaxed),
    };
    counter
        .last_seen
        .store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
}

/// `(detected, enforced)` for a class since process start.
#[cfg(test)]
pub(crate) fn counts(class: Class) -> (u64, u64) {
    let counter = &COUNTERS[class.index()];
    (
        counter.detected.load(Ordering::Relaxed),
        counter.enforced.load(Ordering::Relaxed),
    )
}

pub(crate) fn snapshot(config: &Config) -> Value {
    let rows: Vec<Value> = Class::ALL
        .iter()
        .map(|class| {
            let counter = &COUNTERS[class.index()];
            let last = counter.last_seen.load(Ordering::Relaxed);
            json!({
                "class": class.key(),
                "mode": config.mode(*class).as_str(),
                "detected": counter.detected.load(Ordering::Relaxed),
                "enforced": counter.enforced.load(Ordering::Relaxed),
                "lastSeen": (last > 0).then_some(last),
            })
        })
        .collect();
    json!({ "classes": rows, "scope": "since-process-start" })
}

/// At most one audit row per class and source a minute, and a bounded map, so a
/// scanner cannot turn the audit log into a write amplifier.
pub(crate) fn audit_due(class: Class, source: Option<IpAddr>) -> bool {
    static SEEN: Mutex<Option<HashMap<(Class, Option<IpAddr>), Instant>>> = Mutex::new(None);
    let mut guard = SEEN.lock().unwrap_or_else(|p| p.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    let now = Instant::now();
    if map.len() >= 4096 {
        map.retain(|_, at| now.duration_since(*at) < Duration::from_secs(60));
        if map.len() >= 4096 {
            return false;
        }
    }
    match map.get(&(class, source)) {
        Some(at) if now.duration_since(*at) < Duration::from_secs(60) => false,
        _ => {
            map.insert((class, source), now);
            true
        }
    }
}
