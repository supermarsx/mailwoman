#![forbid(unsafe_code)]
//! Layered cache for Mailwoman V6 (SPEC §15.6, plan §2.2): a per-class matrix
//! over an in-process memory layer (`moka`) → Redis/Valkey (`fred`, rustls TLS,
//! optional) → the store (`mw-store`) fall-through.
//!
//! **Frozen contract (§2.2):** [`Cache::get`]/[`Cache::set`]/[`Cache::invalidate`]
//! `<T>(class, key, loader)`; the [`CacheClass`] rows are the SPEC §15.6
//! scope-matrix classes; the [`PlaintextDerived`] marker gates Redis/memory
//! placement for zero-access accounts *structurally* (the cache refuses to
//! persist a plaintext-derived value from a zero-access account beyond
//! per-request scope — enforced by the type, not operator diligence);
//! [`Cache::posture`] returns the effective matrix for `mailwoman doctor`.
//!
//! **Never authoritative.** Redis is optional and never the source of truth:
//! no Redis configured, or Redis down, degrades to memory + store + loader with
//! no error and no data loss (SPEC §15, plan §3 e2).
//!
//! **Structural zero-access exclusion.** The only methods that accept a
//! [`PlaintextDerived`] value ([`Cache::get_derived`]/[`Cache::set_derived`])
//! route it through the exclusion check: for a zero-access account such a value
//! is *forced to per-request scope* — it is never written to the memory, Redis,
//! or store cache tiers. The placement is gated by the value's type + the
//! account posture, not by operator configuration.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use moka::Expiry;
use moka::future::Cache as MokaCache;
use serde::{Deserialize, Serialize};

use fred::clients::Client as FredClient;
use fred::prelude::{
    Builder, ClientLike, Config as FredConfig, Error as FredError, Expiration, KeysInterface,
};

use bytes::Bytes;
use mw_store::Store;

// ─── Frozen §2.2 public types ────────────────────────────────────────────────

/// The SPEC §15.6 cache classes (each has its own layer set + TTL in the matrix).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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

impl CacheClass {
    /// Every class, in matrix order.
    pub const ALL: [CacheClass; 8] = [
        CacheClass::Sessions,
        CacheClass::HeaderWindows,
        CacheClass::MessageBodies,
        CacheClass::Blobs,
        CacheClass::SearchHotSet,
        CacheClass::PushPresence,
        CacheClass::RateLimit,
        CacheClass::GalDirectory,
    ];

    /// Stable short slug used to namespace cache keys across layers.
    pub fn as_str(self) -> &'static str {
        match self {
            CacheClass::Sessions => "sessions",
            CacheClass::HeaderWindows => "header-windows",
            CacheClass::MessageBodies => "message-bodies",
            CacheClass::Blobs => "blobs",
            CacheClass::SearchHotSet => "search-hot-set",
            CacheClass::PushPresence => "push-presence",
            CacheClass::RateLimit => "rate-limit",
            CacheClass::GalDirectory => "gal-directory",
        }
    }

    /// Whether SPEC §15.6 permits this class in the Redis tier at all. `Blobs`
    /// are content-addressed on disk/S3 and are **never** Redis-eligible; every
    /// other class may be assigned Redis by an admin override (message bodies
    /// only as encrypted values — opt-in).
    pub fn redis_eligible(self) -> bool {
        !matches!(self, CacheClass::Blobs)
    }
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

/// Per-class layer + TTL policy (SPEC §15.6). Admin-overridable; the defaults
/// are supplied by [`ScopeMatrix::spec_defaults`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassPolicy {
    pub class: CacheClass,
    pub layers: Vec<CacheLayer>,
    pub ttl_secs: u64,
}

impl ClassPolicy {
    fn has(&self, layer: CacheLayer) -> bool {
        self.layers.contains(&layer)
    }
}

/// The effective cache posture (per-class policy set) surfaced by
/// `mailwoman doctor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePosture {
    pub classes: Vec<ClassPolicy>,
    /// Whether a Redis/Valkey layer is configured at all.
    pub redis_configured: bool,
    /// Whether the configured Redis/Valkey layer is currently reachable.
    pub redis_connected: bool,
    /// Whether a durable `mw-store` fall-through tier is attached.
    pub store_attached: bool,
}

/// The scope-matrix configuration (SPEC §15.6). [`ScopeMatrix::spec_defaults`]
/// supplies the SPEC "Default" column; [`ScopeMatrix::apply_override`] merges an
/// admin override for one class.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeMatrix {
    pub classes: Vec<ClassPolicy>,
}

impl ScopeMatrix {
    /// The SPEC §15.6 "Default" column. Redis is opt-in per class, so no class
    /// ships with a Redis tier by default; an admin adds it via
    /// [`ScopeMatrix::apply_override`] (rejected for non-Redis-eligible classes).
    pub fn spec_defaults() -> Self {
        use CacheClass::*;
        use CacheLayer::{Memory, Store};
        let p = |class, layers: &[CacheLayer], ttl_secs| ClassPolicy {
            class,
            layers: layers.to_vec(),
            ttl_secs,
        };
        ScopeMatrix {
            classes: vec![
                p(Sessions, &[Memory, Store], 3_600),
                p(HeaderWindows, &[Memory], 300),
                p(MessageBodies, &[Store], 86_400),
                p(Blobs, &[Store], 604_800),
                p(SearchHotSet, &[Memory], 120),
                p(PushPresence, &[Memory], 60),
                p(RateLimit, &[Memory], 60),
                p(GalDirectory, &[Memory, Store], 3_600),
            ],
        }
    }

    fn policy(&self, class: CacheClass) -> Option<&ClassPolicy> {
        self.classes.iter().find(|c| c.class == class)
    }

    /// Merge an admin override for a single class. A Redis tier requested for a
    /// class that is not Redis-eligible (SPEC §15.6: `Blobs`) is dropped and the
    /// dropped layer is returned to the caller for reporting.
    pub fn apply_override(&mut self, mut policy: ClassPolicy) -> Vec<CacheLayer> {
        let mut dropped = Vec::new();
        if !policy.class.redis_eligible() && policy.layers.contains(&CacheLayer::Redis) {
            policy.layers.retain(|l| *l != CacheLayer::Redis);
            dropped.push(CacheLayer::Redis);
        }
        match self.classes.iter_mut().find(|c| c.class == policy.class) {
            Some(existing) => *existing = policy,
            None => self.classes.push(policy),
        }
        dropped
    }
}

/// Marker wrapping a value derived from account plaintext. The cache layer
/// REFUSES to place a `PlaintextDerived` value from a zero-access account into
/// Redis, memory, or the store cache tier beyond per-request scope — enforced by
/// the type (§2.2), not by operator diligence. Only [`Cache::get_derived`] and
/// [`Cache::set_derived`] accept it, and both consult the zero-access posture.
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

/// The posture of the account a cache operation is performed on behalf of.
///
/// This is the switch that makes the zero-access exclusion *structural*: a
/// [`PlaintextDerived`] value tagged [`AccountPosture::ZeroAccess`] can never be
/// written to a shared cache tier — the type + this posture gate the placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountPosture {
    /// A conventional account: plaintext-derived values follow the class matrix.
    Standard,
    /// A zero-access account: plaintext-derived values are forced to per-request
    /// scope — never memory, Redis, or store cache tiers.
    ZeroAccess,
}

impl AccountPosture {
    fn is_zero_access(self) -> bool {
        matches!(self, AccountPosture::ZeroAccess)
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

// ─── Configuration ───────────────────────────────────────────────────────────

/// Construction-time configuration for a [`Cache`].
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// The scope matrix. Empty ⇒ [`ScopeMatrix::spec_defaults`] is used.
    pub matrix: ScopeMatrix,
    /// Redis/Valkey connection URL (`redis://…` / `rediss://…` for rustls TLS).
    /// `None` ⇒ the Redis tier is disabled and the cache degrades to memory +
    /// store with no behavior loss.
    pub redis_url: Option<String>,
    /// Maximum number of entries the in-process memory layer holds.
    pub memory_capacity: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            matrix: ScopeMatrix::default(),
            redis_url: None,
            memory_capacity: 10_000,
        }
    }
}

// ─── Memory layer entry + per-entry TTL ──────────────────────────────────────

#[derive(Clone)]
struct MemEntry {
    bytes: Arc<Vec<u8>>,
    ttl: Duration,
}

struct PerEntryExpiry;

impl Expiry<String, MemEntry> for PerEntryExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &MemEntry,
        _created_at: Instant,
    ) -> Option<Duration> {
        if value.ttl.is_zero() {
            None
        } else {
            Some(value.ttl)
        }
    }
}

// ─── The cache ───────────────────────────────────────────────────────────────

/// The layered cache handle (memory → Redis/Valkey → store).
#[derive(Clone)]
pub struct Cache {
    inner: Arc<Inner>,
}

struct Inner {
    matrix: RwLock<ScopeMatrix>,
    memory: MokaCache<String, MemEntry>,
    redis: Option<FredClient>,
    redis_configured: bool,
    redis_connected: AtomicBool,
    store: Option<Store>,
}

impl Default for Cache {
    /// A memory-only cache with the SPEC defaults and no Redis / no store — the
    /// safe degraded posture. Real deployments use [`Cache::connect`].
    fn default() -> Self {
        Self::in_memory(ScopeMatrix::spec_defaults())
    }
}

impl Cache {
    /// Build a memory-only cache (no Redis, no store fall-through) — the shape a
    /// unit test or a Redis-less deployment uses.
    pub fn in_memory(matrix: ScopeMatrix) -> Self {
        Self::build(matrix, None, false, None, 10_000)
    }

    /// Build a cache with an attached [`Store`] fall-through but no Redis.
    pub fn with_store(matrix: ScopeMatrix, store: Store) -> Self {
        Self::build(matrix, None, false, Some(store), 10_000)
    }

    /// Connect the full stack: memory → Redis/Valkey (if `config.redis_url` is
    /// set and reachable) → store. **Never fails on Redis:** a missing or
    /// unreachable Redis is logged and the cache degrades to memory + store.
    pub async fn connect(config: CacheConfig, store: Option<Store>) -> Self {
        let matrix = if config.matrix.classes.is_empty() {
            ScopeMatrix::spec_defaults()
        } else {
            config.matrix
        };

        let (redis, configured, connected) = match &config.redis_url {
            None => {
                tracing::info!(
                    "mw-cache: no Redis/Valkey configured — memory + store only (never authoritative)"
                );
                (None, false, false)
            }
            Some(url) => match Self::try_connect_redis(url).await {
                Ok(client) => {
                    tracing::info!(target: "mw_cache", "connected to Redis/Valkey cache tier");
                    (Some(client), true, true)
                }
                Err(e) => {
                    tracing::warn!(
                        target: "mw_cache",
                        error = %e,
                        "Redis/Valkey unreachable — degrading to memory + store (no data loss)"
                    );
                    (None, true, false)
                }
            },
        };

        debug_assert_eq!(connected, redis.is_some());
        Self::build(matrix, redis, configured, store, config.memory_capacity)
    }

    fn build(
        matrix: ScopeMatrix,
        redis: Option<FredClient>,
        redis_configured: bool,
        store: Option<Store>,
        capacity: u64,
    ) -> Self {
        let memory = MokaCache::builder()
            .max_capacity(capacity)
            .expire_after(PerEntryExpiry)
            .build();
        let connected = redis.is_some();
        Self {
            inner: Arc::new(Inner {
                matrix: RwLock::new(matrix),
                memory,
                redis,
                redis_configured,
                redis_connected: AtomicBool::new(connected),
                store,
            }),
        }
    }

    async fn try_connect_redis(url: &str) -> Result<FredClient, FredError> {
        let config = FredConfig::from_url(url)?;
        let client = Builder::from_config(config).build()?;
        // `init()` resolves once the first connection is established; awaiting it
        // surfaces a connect failure so we can degrade instead of holding a
        // client that will never reach the server.
        client.init().await?;
        Ok(client)
    }

    // ── Matrix / posture ─────────────────────────────────────────────────────

    fn policy(&self, class: CacheClass) -> ClassPolicy {
        let guard = self.inner.matrix.read().expect("matrix lock");
        guard.policy(class).cloned().unwrap_or_else(|| ClassPolicy {
            class,
            layers: vec![CacheLayer::Store],
            ttl_secs: 0,
        })
    }

    /// Apply an admin override for one class at runtime (SPEC §15.6:
    /// admin-configurable). Returns any layers dropped as ineligible.
    pub fn apply_override(&self, policy: ClassPolicy) -> Vec<CacheLayer> {
        let mut guard = self.inner.matrix.write().expect("matrix lock");
        guard.apply_override(policy)
    }

    /// The effective per-class matrix for `mailwoman doctor`.
    pub fn posture(&self) -> CachePosture {
        let guard = self.inner.matrix.read().expect("matrix lock");
        CachePosture {
            classes: guard.classes.clone(),
            redis_configured: self.inner.redis_configured,
            redis_connected: self.inner.redis_connected.load(Ordering::Relaxed),
            store_attached: self.inner.store.is_some(),
        }
    }

    /// Whether the Redis/Valkey tier is currently believed reachable.
    pub fn redis_connected(&self) -> bool {
        self.inner.redis_connected.load(Ordering::Relaxed)
    }

    // ── Frozen §2.2 API: non-plaintext values ────────────────────────────────

    /// Read `key` in `class`, populating from `loader` on a miss (cache-aside),
    /// walking memory → Redis → store then back-filling every configured tier.
    ///
    /// This is the path for values that are **not** plaintext-derived; for
    /// plaintext-derived values use [`Cache::get_derived`], which enforces the
    /// zero-access exclusion.
    pub async fn get<T, F, Fut>(&self, class: CacheClass, key: &str, loader: F) -> CacheResult<T>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<T>>,
    {
        self.get_inner(class, key, false, loader).await
    }

    /// Write `value` for `key` in `class` across the class's configured layers.
    pub async fn set<T>(&self, class: CacheClass, key: &str, value: &T) -> CacheResult<()>
    where
        T: Serialize,
    {
        let bytes = serde_json::to_vec(value)?;
        self.write_layers(class, key, bytes, false).await;
        Ok(())
    }

    /// Invalidate `key` in `class` across every layer.
    pub async fn invalidate(&self, class: CacheClass, key: &str) -> CacheResult<()> {
        let policy = self.policy(class);
        let nk = namespaced(class, key);
        self.inner.memory.invalidate(&nk).await;
        if policy.has(CacheLayer::Redis)
            && let Some(client) = &self.inner.redis
            && let Err(e) = client.del::<i64, _>(nk.clone()).await
        {
            self.note_redis_down(&e);
        }
        if policy.has(CacheLayer::Store)
            && let Some(store) = &self.inner.store
        {
            // No `delete_setting` on the frozen store API — write a tombstone the
            // store-tier reader treats as absent.
            let _ = store.set_setting(&store_key(&nk), TOMBSTONE).await;
        }
        Ok(())
    }

    // ── Plaintext-derived values (zero-access exclusion enforced here) ────────

    /// Read a plaintext-derived value. For a [`AccountPosture::ZeroAccess`]
    /// account the shared tiers are bypassed entirely — the value is loaded
    /// per-request and never cached. For a standard account it behaves like
    /// [`Cache::get`].
    pub async fn get_derived<T, F, Fut>(
        &self,
        class: CacheClass,
        key: &str,
        account: AccountPosture,
        loader: F,
    ) -> CacheResult<PlaintextDerived<T>>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<T>>,
    {
        if account.is_zero_access() {
            // Structural exclusion: never touch a shared tier — per-request only.
            let value = loader().await?;
            return Ok(PlaintextDerived(value));
        }
        let value = self.get_inner(class, key, false, loader).await?;
        Ok(PlaintextDerived(value))
    }

    /// Write a plaintext-derived value. For a [`AccountPosture::ZeroAccess`]
    /// account this is a **no-op on every shared tier** — the value stays
    /// per-request. For a standard account it writes the class's tiers.
    pub async fn set_derived<T>(
        &self,
        class: CacheClass,
        key: &str,
        account: AccountPosture,
        value: &PlaintextDerived<T>,
    ) -> CacheResult<()>
    where
        T: Serialize,
    {
        if account.is_zero_access() {
            // The type + posture gate the placement: nothing is written.
            return Ok(());
        }
        let bytes = serde_json::to_vec(&value.0)?;
        self.write_layers(class, key, bytes, false).await;
        Ok(())
    }

    // ── Internals ────────────────────────────────────────────────────────────

    async fn get_inner<T, F, Fut>(
        &self,
        class: CacheClass,
        key: &str,
        _reserved: bool,
        loader: F,
    ) -> CacheResult<T>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<T>>,
    {
        let policy = self.policy(class);
        let nk = namespaced(class, key);

        // memory → Redis → store, in tier order.
        if policy.has(CacheLayer::Memory)
            && let Some(entry) = self.inner.memory.get(&nk).await
            && let Ok(v) = serde_json::from_slice::<T>(&entry.bytes)
        {
            return Ok(v);
        }
        if policy.has(CacheLayer::Redis)
            && let Some(bytes) = self.redis_get(&nk).await
            && let Ok(v) = serde_json::from_slice::<T>(&bytes)
        {
            // Warm the faster tier on the way up.
            if policy.has(CacheLayer::Memory) {
                self.mem_put(&nk, bytes, policy.ttl_secs).await;
            }
            return Ok(v);
        }
        if policy.has(CacheLayer::Store)
            && let Some(bytes) = self.store_get(&nk).await
            && let Ok(v) = serde_json::from_slice::<T>(&bytes)
        {
            if policy.has(CacheLayer::Memory) {
                self.mem_put(&nk, bytes, policy.ttl_secs).await;
            }
            return Ok(v);
        }

        // Full miss — fall through to the authoritative loader, then back-fill.
        let value = loader().await?;
        let bytes = serde_json::to_vec(&value)?;
        self.write_layers(class, key, bytes, false).await;
        Ok(value)
    }

    /// Write `bytes` for `key` in `class` across the configured tiers. When
    /// `zero_access` the shared tiers are skipped (defensive belt-and-braces —
    /// the public entry points already gate this).
    async fn write_layers(&self, class: CacheClass, key: &str, bytes: Vec<u8>, zero_access: bool) {
        if zero_access {
            return;
        }
        let policy = self.policy(class);
        let nk = namespaced(class, key);
        if policy.has(CacheLayer::Memory) {
            self.mem_put(&nk, bytes.clone(), policy.ttl_secs).await;
        }
        if policy.has(CacheLayer::Redis) {
            self.redis_put(&nk, &bytes, policy.ttl_secs).await;
        }
        if policy.has(CacheLayer::Store)
            && let Some(store) = &self.inner.store
            && let Ok(text) = std::str::from_utf8(&bytes)
        {
            let _ = store.set_setting(&store_key(&nk), text).await;
        }
    }

    async fn mem_put(&self, nk: &str, bytes: Vec<u8>, ttl_secs: u64) {
        self.inner
            .memory
            .insert(
                nk.to_string(),
                MemEntry {
                    bytes: Arc::new(bytes),
                    ttl: Duration::from_secs(ttl_secs),
                },
            )
            .await;
    }

    async fn redis_get(&self, nk: &str) -> Option<Vec<u8>> {
        let client = self.inner.redis.as_ref()?;
        match client.get::<Option<Bytes>, _>(nk).await {
            Ok(v) => v.map(|b| b.to_vec()),
            Err(e) => {
                self.note_redis_down(&e);
                None
            }
        }
    }

    async fn redis_put(&self, nk: &str, bytes: &[u8], ttl_secs: u64) {
        let Some(client) = self.inner.redis.as_ref() else {
            return;
        };
        let expire = if ttl_secs == 0 {
            None
        } else {
            Some(Expiration::EX(ttl_secs as i64))
        };
        if let Err(e) = client
            .set::<(), _, _>(nk, bytes.to_vec(), expire, None, false)
            .await
        {
            self.note_redis_down(&e);
        }
    }

    async fn store_get(&self, nk: &str) -> Option<Vec<u8>> {
        let store = self.inner.store.as_ref()?;
        match store.get_setting(&store_key(nk)).await {
            Ok(Some(text)) if text != TOMBSTONE => Some(text.into_bytes()),
            _ => None,
        }
    }

    fn note_redis_down(&self, err: &FredError) {
        // Redis is never authoritative — record the degradation and carry on.
        if self.inner.redis_connected.swap(false, Ordering::Relaxed) {
            tracing::warn!(target: "mw_cache", error = %err, "Redis/Valkey error — degrading to memory + store");
        }
    }

    // ── Inspection (backs the zero-access exclusion tests + doctor) ───────────

    /// Whether the memory tier currently holds a live entry for `key` in
    /// `class`. Used by the structural zero-access exclusion test to prove — via
    /// the cache's own inspection, no live server required — that a zero-access
    /// `PlaintextDerived` value never lands in memory.
    pub fn memory_contains(&self, class: CacheClass, key: &str) -> bool {
        self.inner.memory.contains_key(&namespaced(class, key))
    }

    /// Whether the Redis tier holds `key` in `class`. `Ok(false)` when Redis is
    /// absent/unreachable. Used by the live-Valkey exclusion test.
    pub async fn redis_contains(&self, class: CacheClass, key: &str) -> bool {
        let Some(client) = self.inner.redis.as_ref() else {
            return false;
        };
        let nk = namespaced(class, key);
        match client.exists::<i64, _>(nk).await {
            Ok(n) => n > 0,
            Err(e) => {
                self.note_redis_down(&e);
                false
            }
        }
    }

    /// Force the memory tier to run its pending eviction/expiry tasks — test
    /// hook so TTL-expiry assertions are deterministic.
    pub async fn run_pending_memory_tasks(&self) {
        self.inner.memory.run_pending_tasks().await;
    }
}

const TOMBSTONE: &str = "\u{0}mw-cache:tombstone";

fn namespaced(class: CacheClass, key: &str) -> String {
    format!("{}:{}", class.as_str(), key)
}

fn store_key(nk: &str) -> String {
    format!("mwcache:{nk}")
}

/// Render the effective posture as the aligned table `mailwoman doctor` prints.
pub fn render_posture(posture: &CachePosture) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let redis = if !posture.redis_configured {
        "not configured"
    } else if posture.redis_connected {
        "connected"
    } else {
        "configured, UNREACHABLE (degraded)"
    };
    let _ = writeln!(out, "Cache posture (SPEC §15.6):");
    let _ = writeln!(
        out,
        "  redis/valkey: {redis}    store fall-through: {}",
        if posture.store_attached {
            "attached"
        } else {
            "none"
        }
    );
    let mut sorted: BTreeMap<&str, &ClassPolicy> = BTreeMap::new();
    for c in &posture.classes {
        sorted.insert(c.class.as_str(), c);
    }
    for (name, policy) in sorted {
        let layers = policy
            .layers
            .iter()
            .map(|l| match l {
                CacheLayer::Memory => "memory",
                CacheLayer::Redis => "redis",
                CacheLayer::Store => "store",
            })
            .collect::<Vec<_>>()
            .join("+");
        let layers = if layers.is_empty() {
            "none".to_string()
        } else {
            layers
        };
        let _ = writeln!(out, "  {name:<16} {layers:<18} ttl={}s", policy.ttl_secs);
    }
    out
}

#[cfg(test)]
mod tests;
