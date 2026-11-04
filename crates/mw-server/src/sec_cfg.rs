//! Small cached reader for the `security.*` registry families used by the
//! sign-in spray / spike detectors and the session binding / theft modules.
//!
//! A read of a family is cached for [`TTL`], so the hot paths (every sign-in
//! failure, every cookie request) never wait on the settings tables more than
//! twice a minute. A storage error keeps the last good copy; before any read
//! the registry defaults apply, which are the documented defaults.
use crate::settings_registry;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

const TTL: Duration = Duration::from_secs(15);

#[derive(Default, Debug, Clone)]
pub(crate) struct Values(HashMap<String, Value>);

impl Values {
    pub(crate) fn from_pairs(pairs: impl IntoIterator<Item = (String, Value)>) -> Self {
        Self(pairs.into_iter().collect())
    }
    fn get(&self, key: &str) -> Value {
        self.0.get(key).cloned().unwrap_or_else(|| {
            settings_registry::lookup(key).map_or(Value::Null, |d| d.default_value())
        })
    }
    pub(crate) fn num(&self, key: &str) -> u64 {
        self.get(key).as_u64().unwrap_or(0)
    }
    pub(crate) fn flag(&self, key: &str) -> bool {
        self.get(key).as_bool().unwrap_or(false)
    }
    pub(crate) fn text(&self, key: &str) -> String {
        self.get(key).as_str().unwrap_or_default().to_owned()
    }
    pub(crate) fn list(&self, key: &str) -> Vec<String> {
        self.get(key)
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }
}

type Key = (uuid::Uuid, &'static str);
#[derive(Default)]
struct Cache {
    by_store: HashMap<Key, (Instant, Arc<Values>)>,
    /// The most recent read per prefix, for callers that cannot await.
    latest: HashMap<&'static str, Arc<Values>>,
}
fn cache() -> &'static Mutex<Cache> {
    static C: OnceLock<Mutex<Cache>> = OnceLock::new();
    C.get_or_init(Default::default)
}
fn lock() -> std::sync::MutexGuard<'static, Cache> {
    cache().lock().unwrap_or_else(|p| p.into_inner())
}

/// The global values of every key starting with `prefix` (for example
/// `"security.spray."`). Cached per store for 15 seconds.
pub(crate) async fn values(store: &mw_store::Store, prefix: &'static str) -> Arc<Values> {
    let key = (store.cache_scope(), prefix);
    if let Some((at, v)) = lock().by_store.get(&key)
        && at.elapsed() < TTL
    {
        return v.clone();
    }
    let rows =
        settings_registry::effective_all(store, &Default::default(), |d| d.key.starts_with(prefix))
            .await;
    let fresh = rows
        .ok()
        .map(|rows| Arc::new(Values::from_pairs(rows.into_iter().map(|r| (r.key, r.value)))));
    let mut guard = lock();
    let value = match (fresh, guard.by_store.get(&key)) {
        (Some(v), _) => v,
        (None, Some((_, old))) => old.clone(),
        (None, None) => Arc::new(Values::default()),
    };
    guard.by_store.insert(key, (Instant::now(), value.clone()));
    guard.latest.insert(prefix, value.clone());
    value
}

/// The last value read for `prefix`, or the registry defaults. Never waits.
pub(crate) fn snapshot(prefix: &'static str) -> Arc<Values> {
    lock()
        .latest
        .get(prefix)
        .cloned()
        .unwrap_or_else(|| Arc::new(Values::default()))
}

/// The first 16 bytes of an HMAC-SHA256 under `secret`, as a keyed hash.
pub(crate) fn keyed_hash(secret: &str, domain: &str, value: &str) -> [u8; 16] {
    use hmac::{Hmac, KeyInit, Mac};
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()).expect("any key length");
    mac.update(domain.as_bytes());
    mac.update(&[0]);
    mac.update(value.as_bytes());
    let mut out = [0u8; 16];
    out.copy_from_slice(&mac.finalize().into_bytes()[..16]);
    out
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
