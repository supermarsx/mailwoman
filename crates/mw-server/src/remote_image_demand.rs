//! Bounded counts from authorized signing batches. No message IDs or URLs are
//! retained; a retry subset must not throttle images already loading.
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
const CAPACITY: usize = 1024;
const TTL: Duration = Duration::from_secs(3600);
#[derive(Default)]
struct Demands(HashMap<[u8; 32], (usize, Instant)>);
impl Demands {
    fn record(&mut self, key: [u8; 32], images: usize, now: Instant) {
        if images == 0 {
            return;
        }
        if self.0.len() >= CAPACITY {
            self.0
                .retain(|_, (_, seen)| now.saturating_duration_since(*seen) < TTL);
        }
        if !self.0.contains_key(&key) && self.0.len() >= CAPACITY {
            if let Some(oldest) = self
                .0
                .iter()
                .min_by_key(|(_, (_, seen))| *seen)
                .map(|(key, _)| *key)
            {
                self.0.remove(&oldest);
            }
        }
        let item = self.0.entry(key).or_insert((0, now));
        item.0 = item.0.max(images.min(512));
        item.1 = now;
    }
    fn count(&mut self, key: &[u8; 32], now: Instant) -> Option<usize> {
        match self.0.get(key) {
            Some((count, seen)) if now.saturating_duration_since(*seen) < TTL => Some(*count),
            Some(_) => {
                self.0.remove(key);
                None
            }
            None => None,
        }
    }
}
fn demands() -> &'static Mutex<Demands> {
    static DEMANDS: OnceLock<Mutex<Demands>> = OnceLock::new();
    DEMANDS.get_or_init(|| Mutex::new(Demands::default()))
}
pub(crate) fn record(key: [u8; 32], images: usize) {
    demands()
        .lock()
        .expect("image demand lock")
        .record(key, images, Instant::now());
}
pub(crate) fn count(key: &[u8; 32]) -> Option<usize> {
    demands()
        .lock()
        .expect("image demand lock")
        .count(key, Instant::now())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retry_subsets_preserve_demand_and_entries_expire() {
        let mut demands = Demands::default();
        let now = Instant::now();
        let key = [1; 32];
        demands.record(key, 96, now);
        demands.record(key, 3, now + Duration::from_secs(1));
        assert_eq!(demands.count(&key, now + Duration::from_secs(2)), Some(96));
        assert_eq!(demands.count(&[2; 32], now), None);
        assert_eq!(
            demands.count(&key, now + TTL + Duration::from_secs(2)),
            None
        );
    }
    #[test]
    fn capacity_and_counts_remain_bounded_without_retaining_identifiers() {
        let mut demands = Demands::default();
        let now = Instant::now();
        for index in 0..CAPACITY + 1 {
            let mut key = [0; 32];
            key[..8].copy_from_slice(&(index as u64).to_le_bytes());
            demands.record(key, usize::MAX, now + Duration::from_millis(index as u64));
        }
        assert_eq!(demands.0.len(), CAPACITY);
        assert!(demands.0.values().all(|(count, _)| *count == 512));
        assert_eq!(demands.count(&[0; 32], now), None);
    }
}
