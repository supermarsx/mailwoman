//! Admission and memoisation for `POST /api/sanitize`.
//!
//! - A semaphore sized from the available parallelism bounds how many sanitise jobs
//!   (render children, or in-process `spawn_blocking` fallbacks) run at once, so a burst
//!   of large bodies queues instead of spawning an unbounded number of processes.
//! - A small byte-bounded LRU, keyed by the SHA-256 of the input and kept in this
//!   process's memory only, answers repeats (re-opens, prefetch then open) without
//!   running the sanitiser again. The output depends on the input alone, so one entry
//!   serves every authenticated caller. Bodies are never logged.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;

/// Largest `html` accepted (bytes); matches the sanitiser's own input limit.
pub(crate) const MAX_HTML_BYTES: usize = mw_sanitize::MAX_INPUT_BYTES;
/// Bytes of sanitised output the cache may hold.
const CACHE_BYTES: usize = 16 * 1024 * 1024;
/// Outputs bigger than this are not cached (one entry must not flush the rest).
const CACHE_ENTRY_MAX: usize = 4 * 1024 * 1024;

type Key = [u8; 32];

#[derive(Default)]
pub(crate) struct Lru {
    map: HashMap<Key, String>,
    order: VecDeque<Key>,
    bytes: usize,
    cap: usize,
}

impl Lru {
    pub(crate) fn with_capacity(cap: usize) -> Lru {
        Lru { cap, ..Lru::default() }
    }
    pub(crate) fn get(&mut self, key: &Key) -> Option<String> {
        let hit = self.map.get(key)?.clone();
        self.order.retain(|k| k != key);
        self.order.push_back(*key);
        Some(hit)
    }
    pub(crate) fn put(&mut self, key: Key, value: String) {
        if value.len() > self.cap.min(CACHE_ENTRY_MAX) {
            return;
        }
        if let Some(old) = self.map.remove(&key) {
            self.bytes -= old.len();
            self.order.retain(|k| k != &key);
        }
        self.bytes += value.len();
        self.map.insert(key, value);
        self.order.push_back(key);
        while self.bytes > self.cap {
            let Some(oldest) = self.order.pop_front() else { break };
            if let Some(v) = self.map.remove(&oldest) {
                self.bytes -= v.len();
            }
        }
    }
    #[cfg(test)]
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }
}

struct Gate {
    cache: Mutex<Lru>,
    permits: Semaphore,
}

fn gate() -> &'static Gate {
    static GATE: OnceLock<Gate> = OnceLock::new();
    GATE.get_or_init(|| {
        let n = std::thread::available_parallelism().map_or(2, usize::from).clamp(2, 8);
        Gate { cache: Mutex::new(Lru::with_capacity(CACHE_BYTES)), permits: Semaphore::new(n) }
    })
}

pub(crate) fn key(html: &str) -> Key {
    Sha256::digest(html.as_bytes()).into()
}

pub(crate) fn cached(key: &Key) -> Option<String> {
    gate().cache.lock().ok()?.get(key)
}

pub(crate) fn store(key: Key, html: &str) {
    if let Ok(mut cache) = gate().cache.lock() {
        cache.put(key, html.to_string());
    }
}

/// Wait for a sanitise slot.
pub(crate) async fn permit() -> tokio::sync::SemaphorePermit<'static> {
    gate().permits.acquire().await.expect("sanitize semaphore is never closed")
}

/// Sanitise in-process on the blocking pool (CPU-bound; keeps async workers free).
pub(crate) async fn sanitize_blocking(html: String) -> String {
    tokio::task::spawn_blocking(move || mw_sanitize::sanitize_email_html(&html))
        .await
        .unwrap_or_else(|_| "<pre data-mw-body-truncated=\"true\"></pre>".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_evicts_oldest_by_bytes_and_refreshes_on_get() {
        let mut lru = Lru::with_capacity(10);
        let (a, b, c) = (key("a"), key("b"), key("c"));
        lru.put(a, "1234".into());
        lru.put(b, "5678".into());
        assert!(lru.get(&a).is_some()); // a is now newest
        lru.put(c, "abcd".into()); // 12 > 10: evict b
        assert!(lru.get(&b).is_none() && lru.get(&a).is_some() && lru.get(&c).is_some());
        assert!(lru.bytes() <= 10 && lru.len() == 2);
        lru.put(key("big"), "x".repeat(11)); // larger than the cache: skipped
        assert_eq!(lru.len(), 2);
    }

    #[tokio::test]
    async fn blocking_sanitize_matches_the_direct_call() {
        let html = "<p onclick=x()>hi<script>1</script></p>";
        assert_eq!(sanitize_blocking(html.into()).await, mw_sanitize::sanitize_email_html(html));
        let _p = permit().await;
    }
}
