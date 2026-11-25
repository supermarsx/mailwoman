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

mod mail_batch;
pub use mail_batch::MAX_MAIL_BATCH_KEYS;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
    memory: RwLock<MemoryLayer>,
    // Mail versions are ephemeral: bounded bytes, bounded expiry, never settings rows.
    mail_memory: RwLock<MemoryLayer>,
    /// The `capacity` both memory layers were built from, to rebuild one.
    capacity: u64,
    /// Test hook: the next memory-layer operation panics.
    #[cfg(test)]
    panic_once: Arc<AtomicBool>,
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
        #[cfg(test)]
        let panic_once = Arc::new(AtomicBool::new(false));
        let layer = |which| MemoryLayer {
            generation: 0,
            cache: new_moka(which, capacity),
        };
        let connected = redis.is_some();
        Self {
            inner: Arc::new(Inner {
                matrix: RwLock::new(matrix),
                memory: RwLock::new(layer(Layer::General)),
                mail_memory: RwLock::new(layer(Layer::Mail)),
                capacity,
                #[cfg(test)]
                panic_once,
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

    /// Evict advisory ephemeral entries without writing persistent tombstones.
    pub async fn invalidate_ephemeral(&self, class: CacheClass, key: &str) -> CacheResult<()> {
        let policy = self.policy(class);
        let nk = namespaced(class, key);
        let key_ref = nk.as_str();
        self.inner
            .guarded(Layer::General, |cache| async move { cache.invalidate(key_ref).await })
            .await;
        self.inner
            .guarded(Layer::Mail, |cache| async move { cache.invalidate(key_ref).await })
            .await;
        if policy.has(CacheLayer::Redis)
            && let Some(client) = &self.inner.redis
            && let Err(e) = client.del::<i64, _>(nk.clone()).await
        {
            self.note_redis_down(&e);
        }
        Ok(())
    }

    /// Invalidate `key` in `class` across every layer.
    pub async fn invalidate(&self, class: CacheClass, key: &str) -> CacheResult<()> {
        self.invalidate_ephemeral(class, key).await?;
        let policy = self.policy(class);
        let nk = namespaced(class, key);
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

    /// Cache mail bytes only after the caller has obtained a fresh authoritative
    /// Message snapshot and built its account/revision or immutable-blob key.
    /// Database validation errors must be handled before calling this method;
    /// Redis availability never substitutes for that validation. Missing values,
    /// auth sessions and zero-access plaintext never enter shared tiers.
    pub async fn get_mail_bytes<F, Fut>(
        &self,
        class: CacheClass,
        key: &str,
        account: AccountPosture,
        loader: F,
    ) -> CacheResult<PlaintextDerived<Option<Vec<u8>>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<Option<Vec<u8>>>>,
    {
        let Some((memory, redis, ttl)) = self.mail_placement(class, account) else {
            return loader().await.map(PlaintextDerived);
        };
        let nk = namespaced(class, key);
        if memory && let Some(entry) = self.inner.memory_get(Layer::Mail, &nk).await {
            return Ok(PlaintextDerived(Some((*entry.bytes).clone())));
        }
        if redis && let Some(sealed) = self.redis_get(&nk).await {
            if let Some(bytes) = self.open_mail_value(&nk, &sealed) {
                if memory {
                    self.mail_put(&nk, bytes.clone(), ttl).await;
                }
                return Ok(PlaintextDerived(Some(bytes)));
            }
        }
        let value = loader().await?;
        if let Some(bytes) = &value
            && bytes.len() <= MAIL_ENTRY_BYTES
        {
            if memory {
                self.mail_put(&nk, bytes.clone(), ttl).await;
            }
            if redis && let Some(sealed) = self.seal_mail_value(&nk, bytes) {
                self.redis_put(&nk, &sealed, ttl).await;
            }
        }
        Ok(PlaintextDerived(value))
    }
    fn seal_mail_value(&self, namespace: &str, bytes: &[u8]) -> Option<Vec<u8>> {
        if bytes.len() > MAIL_ENTRY_BYTES {
            return None;
        }
        self.inner
            .store
            .as_ref()?
            .seal_mail_cache(namespace, bytes)
            .ok()
    }
    fn open_mail_value(&self, namespace: &str, sealed: &[u8]) -> Option<Vec<u8>> {
        // MWB1 + 24-byte nonce + 16-byte tag. Reject oversized values before
        // allocating plaintext; corrupt/foreign envelopes are ordinary misses.
        if sealed.len() > MAIL_ENTRY_BYTES + 44 {
            return None;
        }
        self.inner
            .store
            .as_ref()?
            .open_mail_cache(namespace, sealed)
            .ok()
    }
    async fn mail_put(&self, key: &str, bytes: Vec<u8>, ttl: u64) {
        self.inner.memory_put(Layer::Mail, key, bytes, ttl).await;
    }
    /// Bounded mail-cache occupancy, in bytes, after pending eviction tasks run.
    pub fn mail_memory_bytes(&self) -> u64 {
        self.inner.layer(Layer::Mail).1.weighted_size()
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
            && let Some(entry) = self.inner.memory_get(Layer::General, &nk).await
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
        self.inner.memory_put(Layer::General, nk, bytes, ttl_secs).await;
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
        self.inner.layer(Layer::General).1.contains_key(&namespaced(class, key))
            || self.inner.layer(Layer::Mail).1.contains_key(&namespaced(class, key))
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
        self.inner
            .guarded(Layer::General, |cache| async move { cache.run_pending_tasks().await })
            .await;
        self.inner
            .guarded(Layer::Mail, |cache| async move { cache.run_pending_tasks().await })
            .await;
    }
}

const TOMBSTONE: &str = "\u{0}mw-cache:tombstone";

// ─── Memory layer panic containment (t55) ───────────────────────────────────

/// Which in-process memory layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layer {
    General,
    Mail,
}

impl Layer {
    fn name(self) -> &'static str {
        match self {
            Self::General => "memory",
            Self::Mail => "mail_memory",
        }
    }
}

/// One moka instance and the number of times it was replaced.
struct MemoryLayer {
    generation: u64,
    cache: MokaCache<String, MemEntry>,
}

/// Memory-layer operations that panicked and were contained, since start.
static MEMORY_LAYER_PANICS: AtomicU64 = AtomicU64::new(0);

/// How many memory-layer operations panicked and were contained since the
/// process started (also exported as `mw_cache_memory_panics_total`).
pub fn memory_layer_panics() -> u64 {
    MEMORY_LAYER_PANICS.load(Ordering::Relaxed)
}

fn new_moka(which: Layer, capacity: u64) -> MokaCache<String, MemEntry> {
    let builder = match which {
        Layer::General => MokaCache::builder().max_capacity(capacity),
        Layer::Mail => {
            let minimum_entry_weight = MAIL_MEMORY_BYTES.div_ceil(capacity.max(1)).max(1) as u32;
            MokaCache::builder()
                .max_capacity(if capacity == 0 { 0 } else { MAIL_MEMORY_BYTES })
                .weigher(move |key: &String, value: &MemEntry| {
                    ((key.len() + value.bytes.len()).min(u32::MAX as usize) as u32)
                        .max(minimum_entry_weight)
                })
        }
    }
    .expire_after(PerEntryExpiry);
    builder.build()
}

impl Inner {
    fn slot(&self, which: Layer) -> &RwLock<MemoryLayer> {
        match which {
            Layer::General => &self.memory,
            Layer::Mail => &self.mail_memory,
        }
    }

    /// The current moka instance of `which` (a cheap handle clone).
    fn layer(&self, which: Layer) -> (u64, MokaCache<String, MemEntry>) {
        let guard = self.slot(which).read().unwrap_or_else(std::sync::PoisonError::into_inner);
        (guard.generation, guard.cache.clone())
    }

    /// Run one memory-layer operation, containing a panic inside it.
    ///
    /// moka keeps its maintenance task in a `futures::Shared` while it runs;
    /// when that task panics (moka 0.12.15's frequency sketch overflowed on a
    /// production load), the stored Shared stays poisoned and every later
    /// operation that runs maintenance panics again ("inner future panicked
    /// during poll"), for the life of the process. So a panic here is caught,
    /// logged and counted, the operation answers as a miss (`None`; the memory
    /// layer is advisory), and the instance is replaced by a fresh, empty one
    /// unless another caller already replaced it.
    async fn guarded<T, F, Fut>(&self, which: Layer, op: F) -> Option<T>
    where
        F: FnOnce(MokaCache<String, MemEntry>) -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let (generation, cache) = self.layer(which);
        #[cfg(test)]
        let armed = self.panic_once.swap(false, Ordering::SeqCst);
        let mut fut = std::pin::pin!(async move {
            // Test hook: stands in for a panic inside moka (its maintenance
            // runs inside these operations).
            #[cfg(test)]
            if armed {
                panic!("injected panic in a memory-layer operation");
            }
            op(cache).await
        });
        let caught = std::future::poll_fn(|cx| {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fut.as_mut().poll(cx))) {
                Ok(std::task::Poll::Pending) => std::task::Poll::Pending,
                Ok(std::task::Poll::Ready(value)) => std::task::Poll::Ready(Ok(value)),
                Err(payload) => std::task::Poll::Ready(Err(payload)),
            }
        })
        .await;
        match caught {
            Ok(value) => Some(value),
            Err(payload) => {
                self.contain_panic(which, generation, payload.as_ref());
                None
            }
        }
    }

    fn contain_panic(&self, which: Layer, generation: u64, payload: &(dyn std::any::Any + Send)) {
        MEMORY_LAYER_PANICS.fetch_add(1, Ordering::Relaxed);
        metrics::counter!("mw_cache_memory_panics_total", "layer" => which.name()).increment(1);
        let mut slot = self.slot(which).write().unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot.generation != generation {
            return; // Already replaced after this instance failed.
        }
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-text panic payload".into());
        tracing::error!(
            target: "mw_cache",
            layer = which.name(),
            generation,
            panic = %message,
            "in-process cache layer panicked; replaced with an empty instance"
        );
        slot.cache = new_moka(which, self.capacity);
        slot.generation = generation + 1;
    }

    async fn memory_get(&self, which: Layer, key: &str) -> Option<MemEntry> {
        self.guarded(which, |cache| async move { cache.get(key).await })
            .await
            .flatten()
    }

    async fn memory_put(&self, which: Layer, key: &str, bytes: Vec<u8>, ttl_secs: u64) {
        let entry = MemEntry {
            bytes: Arc::new(bytes),
            ttl: Duration::from_secs(ttl_secs),
        };
        self.guarded(which, |cache| async move { cache.insert(key.to_owned(), entry).await })
            .await;
    }
}

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

const MAIL_MEMORY_BYTES: u64 = 64 * 1024 * 1024;
const MAIL_ENTRY_BYTES: usize = 8 * 1024 * 1024;
/// Account and message revision are unambiguously encoded; legacy stable-ID keys
/// cannot collide with this namespace. The revision comes only from the store.
pub fn envelope_cache_key(account: &str, stable_id: &str, revision: &str) -> String {
    format!(
        "mail-v2:envelope:{}",
        serde_json::to_string(&(account, stable_id, revision)).expect("string tuple")
    )
}
/// Bodies are immutable, account-scoped content addresses rather than message IDs.
pub fn body_cache_key(account: &str, blob_ref: &str) -> String {
    format!(
        "mail-v2:body:{}",
        serde_json::to_string(&(account, blob_ref)).expect("string tuple")
    )
}

#[cfg(test)]
#[path = "panic_tests.rs"]
mod panic_tests;
