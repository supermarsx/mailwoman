//! Bounded, account- and route-separated image miss coordination. No URL or
//! account identifier is retained. Authorization happens before admission.
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock, Weak},
    time::Instant,
};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

const CAPACITY: usize = 1024;
#[derive(Default)]
struct Flights(Mutex<HashMap<[u8; 32], Weak<AsyncMutex<()>>>>);

impl Flights {
    fn slot(&self, account: &str, route: &str, url: &str) -> Option<Arc<AsyncMutex<()>>> {
        let mut hash = Sha256::new();
        for part in [account, route, url] {
            hash.update((part.len() as u64).to_be_bytes());
            hash.update(part.as_bytes());
        }
        let key: [u8; 32] = hash.finalize().into();
        let mut entries = self.0.lock().expect("image flight lock");
        if let Some(slot) = entries.get(&key).and_then(Weak::upgrade) {
            return Some(slot);
        }
        if entries.len() >= CAPACITY {
            entries.retain(|_, slot| slot.strong_count() > 0);
            if entries.len() >= CAPACITY {
                return None;
            }
        }
        let slot = Arc::new(AsyncMutex::new(()));
        entries.insert(key, Arc::downgrade(&slot));
        Some(slot)
    }
}

/// Guard spans cache recheck, fetch, decode and cache publication. Cancellation
/// releases ownership automatically; a waiter never gets a fresh total budget.
pub(crate) async fn acquire(
    account: &str,
    route: &str,
    url: &str,
    deadline: Instant,
) -> Option<OwnedMutexGuard<()>> {
    if Instant::now() >= deadline {
        return None;
    }
    static FLIGHTS: OnceLock<Flights> = OnceLock::new();
    let slot = FLIGHTS
        .get_or_init(Flights::default)
        .slot(account, route, url)?;
    let guard =
        tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), slot.lock_owned())
            .await
            .ok()?;
    (Instant::now() < deadline).then_some(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capacity_sheds_new_keys_without_splitting_active_flights_and_reclaims_idle() {
        let flights = Flights::default();
        let mut held = Vec::new();
        for n in 0..CAPACITY {
            held.push(flights.slot("account", "route", &n.to_string()).unwrap());
        }
        assert!(flights.slot("account", "route", "overflow").is_none());
        assert!(Arc::ptr_eq(
            &held[0],
            &flights.slot("account", "route", "0").unwrap()
        ));
        held.clear();
        assert!(flights.slot("account", "route", "overflow").is_some());
        assert_eq!(flights.0.lock().unwrap().len(), 1);
    }
}
