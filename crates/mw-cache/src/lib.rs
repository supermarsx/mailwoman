#![forbid(unsafe_code)]
// SCAFFOLD (t6-e0): stub crate — the frozen §2.2 API names exist so the Batch-B
// executors (e2, e10) compile against them; e2 owns the real implementation.
#![allow(dead_code, clippy::unused_async)]
//! Layered cache for Mailwoman V6 (SPEC §15.6, plan §2.2): a per-class matrix
//! over an in-process memory layer (`moka`) → Redis/Valkey (`fred`) → the store.
//!
//! **Frozen contract (§2.2):** `Cache::get/set/invalidate<T>(class, key, loader)`;
//! the [`CacheClass`] rows are the SPEC §15.6 scope-matrix classes; the
//! [`PlaintextDerived`] marker gates Redis/memory placement for zero-access
//! accounts *structurally* (the cache refuses to persist a plaintext-derived
//! value from a zero-access account beyond per-request scope — enforced by the
//! type, not operator diligence); [`Cache::posture`] returns the effective
//! matrix for `mailwoman doctor`.
//!
//! Losing Redis loses performance, never data — the cache is never authoritative.
//!
//! e2 fills the bodies (currently `unimplemented!()`), links `fred`/`moka`, and
//! threads the `mw-store` handle for the store fall-through.

use serde::{Deserialize, Serialize};

/// The SPEC §15.6 cache classes (each has its own layer set + TTL in the matrix).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheClass {
    Sessions,
    HeaderWindows,
    MessageBodies,
    Blobs,
    SearchHotSet,
    PushPresence,
    RateLimit,
    GalDirectory,
}

/// A cache tier a class may be placed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheLayer {
    /// In-process `moka` layer (per-request scope for zero-access plaintext).
    Memory,
    /// Redis/Valkey via `fred` — never holds a zero-access `PlaintextDerived` value.
    Redis,
    /// The authoritative `mw-store` fall-through.
    Store,
}

/// Per-class layer + TTL policy (SPEC §15.6). Admin-overridable; e2 supplies the
/// SPEC defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassPolicy {
    pub class: CacheClass,
    pub layers: Vec<CacheLayer>,
    pub ttl_secs: u64,
}

/// The effective cache posture (per-class policy set) surfaced by
/// `mailwoman doctor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePosture {
    pub classes: Vec<ClassPolicy>,
    /// Whether a live Redis/Valkey layer is currently reachable.
    pub redis_connected: bool,
}

/// The scope-matrix configuration (SPEC §15.6). e2 supplies the defaults + the
/// admin override merge.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeMatrix {
    pub classes: Vec<ClassPolicy>,
}

/// Marker wrapping a value derived from account plaintext. The cache layer
/// REFUSES to place a `PlaintextDerived` value from a zero-access account into
/// Redis or the memory layer beyond per-request scope — enforced by the type
/// (§2.2), not by operator diligence.
#[derive(Debug, Clone)]
pub struct PlaintextDerived<T>(pub T);

impl<T> PlaintextDerived<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("redis/valkey error: {0}")]
    Redis(String),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("store error: {0}")]
    Store(String),
}

pub type CacheResult<T> = Result<T, CacheError>;

/// The layered cache handle (memory → Redis/Valkey → store).
///
/// STUB: e2 replaces this with the real moka/fred/store fields + config.
#[derive(Clone, Default)]
pub struct Cache {
    _private: (),
}

impl Cache {
    /// Read `key` in `class`, populating from `loader` on a miss (cache-aside).
    pub async fn get<T, F, Fut>(&self, _class: CacheClass, _key: &str, _loader: F) -> CacheResult<T>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<T>>,
    {
        unimplemented!("mw-cache::Cache::get — filled by t6-e2")
    }

    /// Write `value` for `key` in `class` across the class's configured layers.
    pub async fn set<T>(&self, _class: CacheClass, _key: &str, _value: &T) -> CacheResult<()>
    where
        T: Serialize,
    {
        unimplemented!("mw-cache::Cache::set — filled by t6-e2")
    }

    /// Invalidate `key` in `class` across every layer.
    pub async fn invalidate(&self, _class: CacheClass, _key: &str) -> CacheResult<()> {
        unimplemented!("mw-cache::Cache::invalidate — filled by t6-e2")
    }

    /// The effective per-class matrix for `mailwoman doctor`.
    pub fn posture(&self) -> CachePosture {
        unimplemented!("mw-cache::Cache::posture — filled by t6-e2")
    }
}
