//! The per-store host state + the capability-gated host-import implementation
//! (plan §2.1, SPEC §7.1/§7.5). This is the trust boundary: **every** host import
//! a guest can reach is gated here against the plugin's admin-approved grant
//! (capabilities + `net_allowlist`) — deny-by-default, no ambient authority.

use std::collections::BTreeSet;
use std::sync::Arc;

use async_trait::async_trait;
use wasmtime::component::{Linker, ResourceTable};
use wasmtime::{ResourceLimiter, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::bindings::mailwoman::plugin::host::{self, Host, HttpRequest, HttpResponse, LogLevel};
use crate::bindings::mailwoman::plugin::types::PluginError as WitError;
use crate::{Capability, PluginError, PluginLimits, Result};

const MIB: usize = 1024 * 1024;

// ── Injected host services (embedder-provided; deny-leaning defaults) ──────────

/// A host-mediated outbound HTTP request (the guest never opens a socket).
#[derive(Debug, Clone)]
pub struct HttpReq {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

/// The response returned to the guest.
#[derive(Debug, Clone)]
pub struct HttpResp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Outbound HTTP, performed by the host over its own (rustls) client. The host
/// enforces the `net_allowlist` *before* calling this, so an implementation only
/// ever sees already-authorized requests. `mw-server` injects a `reqwest` impl at
/// mount (e14); the default refuses (no network configured).
#[async_trait]
pub trait HttpFetcher: Send + Sync {
    async fn fetch(&self, req: HttpReq) -> std::result::Result<HttpResp, String>;
}

/// Acquires a short-lived OAuth token for a bound account. The host holds/refreshes
/// long-lived secrets; the guest never sees them (plan §2.1).
#[async_trait]
pub trait OAuthTokenProvider: Send + Sync {
    async fn token(&self, account: &str) -> std::result::Result<String, String>;
}

/// A per-plugin scoped KV scratch store.
#[async_trait]
pub trait KvStore: Send + Sync {
    async fn get(&self, namespaced_key: &str) -> Option<Vec<u8>>;
    async fn put(&self, namespaced_key: &str, value: Vec<u8>);
}

/// Governable wall-clock source (host-provided ⇒ no ambient WASI clock).
pub trait Clock: Send + Sync {
    fn now_millis(&self) -> u64;
}

/// Governable randomness (host-provided ⇒ no ambient WASI random).
pub trait Rng: Send + Sync {
    fn fill(&self, len: usize) -> Vec<u8>;
}

/// The bundle of host services a [`crate::PluginHost`] provides to its guests.
#[derive(Clone)]
pub struct HostServices {
    pub http: Arc<dyn HttpFetcher>,
    pub oauth: Arc<dyn OAuthTokenProvider>,
    pub kv: Arc<dyn KvStore>,
    pub clock: Arc<dyn Clock>,
    pub rng: Arc<dyn Rng>,
}

impl Default for HostServices {
    fn default() -> Self {
        Self {
            http: Arc::new(DeniedHttp),
            oauth: Arc::new(DeniedOAuth),
            kv: Arc::new(MemoryKv::default()),
            clock: Arc::new(SystemClock),
            rng: Arc::new(OsRng),
        }
    }
}

impl std::fmt::Debug for HostServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HostServices { .. }")
    }
}

struct DeniedHttp;
#[async_trait]
impl HttpFetcher for DeniedHttp {
    async fn fetch(&self, _req: HttpReq) -> std::result::Result<HttpResp, String> {
        Err("no HTTP fetcher configured (host did not inject one)".into())
    }
}

struct DeniedOAuth;
#[async_trait]
impl OAuthTokenProvider for DeniedOAuth {
    async fn token(&self, _account: &str) -> std::result::Result<String, String> {
        Err("no OAuth provider configured".into())
    }
}

#[derive(Default)]
struct MemoryKv {
    map: tokio::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}
#[async_trait]
impl KvStore for MemoryKv {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.map.lock().await.get(key).cloned()
    }
    async fn put(&self, key: &str, value: Vec<u8>) {
        self.map.lock().await.insert(key.to_string(), value);
    }
}

struct SystemClock;
impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

struct OsRng;
impl Rng for OsRng {
    fn fill(&self, len: usize) -> Vec<u8> {
        let mut buf = vec![0u8; len];
        // getrandom is a CSPRNG; on failure fall back to an empty buffer rather
        // than panicking the host (deny-leaning).
        if getrandom::getrandom(&mut buf).is_err() {
            buf.clear();
        }
        buf
    }
}

// ── Capability gate ───────────────────────────────────────────────────────────

/// The effective, admin-approved grant a store enforces: `manifest ∩ grant`
/// capabilities plus the manifest `net_allowlist`. Nothing here can be widened at
/// runtime (plan §2.1 — the host never grants beyond this).
#[derive(Debug, Clone, Default)]
pub(crate) struct CapGate {
    caps: BTreeSet<Capability>,
    net_allowlist: Vec<String>,
}

impl CapGate {
    pub(crate) fn new(caps: BTreeSet<Capability>, net_allowlist: Vec<String>) -> Self {
        Self {
            caps,
            net_allowlist,
        }
    }

    pub(crate) fn has(&self, c: Capability) -> bool {
        self.caps.contains(&c)
    }

    /// Whether `host` is reachable under the `net_allowlist`. An entry matches by
    /// exact host (case-insensitive) or as a `.suffix` / `*.suffix` domain wildcard.
    pub(crate) fn net_allows(&self, host: &str) -> bool {
        let host = host.trim().to_ascii_lowercase();
        self.net_allowlist.iter().any(|entry| {
            let e = entry.trim().to_ascii_lowercase();
            if let Some(suffix) = e.strip_prefix("*.").or_else(|| e.strip_prefix('.')) {
                host == suffix || host.ends_with(&format!(".{suffix}"))
            } else {
                host == e
            }
        })
    }
}

// ── Resource limiter (memory ceiling; a trip sets `oom` for attribution) ──────

pub(crate) struct Limiter {
    memory_max: usize,
    table_max: usize,
    /// Set when a growth was denied so the caller can attribute a subsequent trap
    /// to the memory ceiling (→ `PluginError::LimitExceeded`) rather than a panic.
    pub(crate) oom: bool,
}

impl Limiter {
    fn new(limits: &PluginLimits) -> Self {
        Self {
            memory_max: (limits.memory_mb as usize) * MIB,
            table_max: 100_000,
            oom: false,
        }
    }
}

impl ResourceLimiter for Limiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        if desired > self.memory_max {
            self.oom = true;
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(desired <= self.table_max)
    }
}

// ── The store data ────────────────────────────────────────────────────────────

/// Per-store host state. Holds the (restricted) WASI ctx, the capability gate, the
/// resource limiter, and the injected services.
pub(crate) struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
    pub(crate) limits: Limiter,
    gate: CapGate,
    services: Arc<HostServices>,
    plugin_id: String,
}

impl HostState {
    pub(crate) fn new(
        gate: CapGate,
        services: Arc<HostServices>,
        limits: &PluginLimits,
        plugin_id: String,
    ) -> Self {
        // A MAXIMALLY-RESTRICTED WASI ctx: no preopened dirs (no filesystem), no
        // env, no args, no inherited stdio, no network. A std guest instantiates,
        // but has ZERO ambient authority — every real capability flows through the
        // gated host imports below (plan §1.1 / §7.5).
        let wasi = WasiCtxBuilder::new().build();
        Self {
            wasi,
            table: ResourceTable::new(),
            limits: Limiter::new(limits),
            gate,
            services,
            plugin_id,
        }
    }
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

fn denied(msg: impl Into<String>) -> WitError {
    WitError::CapabilityDenied(msg.into())
}

/// Extract the host portion of a URL without pulling a URL crate.
fn url_host(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(after_scheme);
    // strip userinfo@ and :port
    let hostport = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let host = hostport.split(':').next().unwrap_or(hostport);
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

// ── The gated host-import implementation ──────────────────────────────────────

// NB: the generated `Host` trait uses native `async fn` in trait (RPITIT), so this
// impl must NOT carry `#[async_trait]` (that would box + change the signatures).
impl Host for HostState {
    async fn http_fetch(
        &mut self,
        req: HttpRequest,
    ) -> wasmtime::Result<std::result::Result<HttpResponse, WitError>> {
        // DENY-BY-DEFAULT: requires the `net` capability AND a URL whose host is in
        // the manifest `net_allowlist`. Either miss ⇒ capability-denied (the guest
        // sees the error; it never reaches the network).
        if !self.gate.has(Capability::Net) {
            return Ok(Err(denied("net capability not granted")));
        }
        let host = match url_host(&req.url) {
            Some(h) => h,
            None => return Ok(Err(WitError::Transport("malformed url".into()))),
        };
        if !self.gate.net_allows(&host) {
            return Ok(Err(denied(format!(
                "host '{host}' is outside the plugin net_allowlist"
            ))));
        }
        let out = self
            .services
            .http
            .fetch(HttpReq {
                method: req.method,
                url: req.url,
                headers: req.headers,
                body: req.body,
            })
            .await;
        Ok(match out {
            Ok(r) => Ok(HttpResponse {
                status: r.status,
                headers: r.headers,
                body: r.body,
            }),
            Err(e) => Err(WitError::Transport(e)),
        })
    }

    async fn oauth_token(
        &mut self,
        account: String,
    ) -> wasmtime::Result<std::result::Result<String, WitError>> {
        // OAuth acquisition is a bridge (account-backend) capability.
        if !self.gate.has(Capability::AccountBackend) {
            return Ok(Err(denied(
                "oauth-token requires the account-backend capability",
            )));
        }
        Ok(self
            .services
            .oauth
            .token(&account)
            .await
            .map_err(WitError::Auth))
    }

    async fn kv_get(&mut self, key: String) -> wasmtime::Result<Option<Vec<u8>>> {
        if !self.gate.has(Capability::StoreKvScoped) {
            return Ok(None);
        }
        Ok(self.services.kv.get(&self.namespaced(&key)).await)
    }

    async fn kv_put(&mut self, key: String, value: Vec<u8>) -> wasmtime::Result<()> {
        if self.gate.has(Capability::StoreKvScoped) {
            let k = self.namespaced(&key);
            self.services.kv.put(&k, value).await;
        }
        Ok(())
    }

    async fn log(&mut self, _level: LogLevel, _message: String) -> wasmtime::Result<()> {
        // No-content logging floor: accepted but never persisted with content here.
        // The embedder may wire a redacting sink; the default drops it.
        Ok(())
    }

    async fn now(&mut self) -> wasmtime::Result<u64> {
        Ok(self.services.clock.now_millis())
    }

    async fn random(&mut self, len: u32) -> wasmtime::Result<Vec<u8>> {
        // Bounded to avoid a guest asking for an enormous host allocation.
        let len = (len as usize).min(4096);
        Ok(self.services.rng.fill(len))
    }
}

impl HostState {
    fn namespaced(&self, key: &str) -> String {
        format!("plugin:{}:{key}", self.plugin_id)
    }
}

// ── Linker + store construction ───────────────────────────────────────────────

/// Build a `Linker` wiring (1) the restricted WASI p2 surface and (2) the gated
/// `mailwoman:plugin/host` imports. Reused across every store of one plugin.
pub(crate) fn build_linker(engine: &wasmtime::Engine) -> Result<Linker<HostState>> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)
        .map_err(|e| PluginError::Load(format!("wasi linker: {e}")))?;
    host::add_to_linker::<HostState, wasmtime::component::HasSelf<HostState>>(&mut linker, |s| s)
        .map_err(|e| PluginError::Load(format!("host linker: {e}")))?;
    Ok(linker)
}

/// Create a fresh store for one plugin invocation/session, applying the memory
/// ceiling, the wall-clock deadline (epoch), and optional fuel.
pub(crate) fn new_store(
    engine: &wasmtime::Engine,
    gate: CapGate,
    services: Arc<HostServices>,
    limits: &PluginLimits,
    plugin_id: String,
) -> Result<Store<HostState>> {
    let mut store = Store::new(engine, HostState::new(gate, services, limits, plugin_id));
    store.limiter(|s| &mut s.limits);
    // Wall-clock deadline via epoch interruption (default: trap on deadline).
    store.set_epoch_deadline(crate::engine::EpochTicker::ticks_for(limits.deadline_ms));
    store.epoch_deadline_trap();
    // The engine has `consume_fuel(true)` globally, so a store starts with 0 fuel
    // unless set — always set it. `None` ⇒ effectively unlimited (the wall-clock
    // deadline is the active bound); `Some(f)` ⇒ a deterministic fuel budget.
    store
        .set_fuel(limits.fuel.unwrap_or(u64::MAX))
        .map_err(|e| PluginError::Load(format!("set fuel: {e}")))?;
    Ok(store)
}

/// Map a failed guest-export call onto a typed [`PluginError`] — a resource-limit
/// trip is `LimitExceeded` (host survives), never a panic (plan §2.1).
pub(crate) fn map_call_err(store: &Store<HostState>, err: wasmtime::Error) -> PluginError {
    if store.data().limits.oom {
        return PluginError::LimitExceeded("linear-memory ceiling exceeded".into());
    }
    if let Some(trap) = err.downcast_ref::<wasmtime::Trap>() {
        return match trap {
            wasmtime::Trap::Interrupt => {
                PluginError::LimitExceeded("wall-clock deadline exceeded".into())
            }
            wasmtime::Trap::OutOfFuel => PluginError::LimitExceeded("fuel exhausted".into()),
            other => PluginError::Runtime(format!("guest trap: {other}")),
        };
    }
    PluginError::Runtime(err.to_string())
}
