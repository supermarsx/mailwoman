//! Anonymizing remote-image proxy + remote-image display grants + the tightened
//! shell CSP constant (t16 26.16 S7/S8/S10, plan §DQ3).
//!
//! # Why a proxy (S7/S8)
//! Email HTML embeds `<img src>` pointing at attacker-controlled hosts. Loading them
//! directly leaks the reader's IP, User-Agent, and open-time to the sender (the
//! classic tracking pixel) and can smuggle requests to internal services. Instead the
//! sanitizer strips remote images by default; when the user grants a scope
//! ([`Store::grant_remote_image`](mw_store::Store)) the web rewrites the granted
//! images to **`GET /api/image-proxy?url=…`**, and THIS server fetches them — so the
//! only host the reader's browser ever contacts is Mailwoman itself.
//!
//! # SSRF hardening (DQ3 — this fetches attacker-controlled URLs; treat as hostile)
//! The deny-by-default egress policy itself lives in the [`mw_egress`] crate as of
//! 26.20 (t22-e6) — it was `pub(crate)` here, which is why the workspace's other
//! egress surfaces could not reuse it. Nothing about it changed in the move:
//!   * scheme ∈ {`http`,`https`} only; URLs carrying credentials are refused;
//!   * DNS is resolved **once, by us**, and the fetch is PINNED to the resolved IP
//!     (reqwest `resolve`) so a name cannot rebind to a new address between our check
//!     and the connect (anti-DNS-rebinding). The client also sets `no_proxy` — an
//!     ambient `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` would otherwise hand the
//!     HOSTNAME to a proxy that resolves it itself, bypassing the pin
//!     ([`mw_egress::harden_client`]);
//!   * every resolved address is checked against [`mw_egress::ip_allowed`] —
//!     loopback, private, link-local (incl. the `169.254.169.254` cloud-metadata
//!     address), CGNAT, unique-local/link-local IPv6, multicast, unspecified and
//!     reserved ranges are REFUSED; IPv4-mapped IPv6 is unwrapped and re-checked;
//!   * redirects are NOT auto-followed — each hop's `Location` is re-parsed and
//!     re-validated through the same gate (a redirect to a private target is refused);
//!   * hard caps bound response size and the per-request timeout.
//!
//! One consequence of the move worth knowing before optimizing: this module no longer
//! builds an HTTP client at all — [`mw_egress::fetch_hop`] builds one per hop, because
//! the pin is `.resolve(host, addr)` baked into the builder and each redirect hop is a
//! different `(host, addr)` discovered only after the previous hop answers. A client
//! cannot be shared across hops without either dropping the pin for hops 2..n or
//! replacing the pinning mechanism with a custom resolver. Any such change belongs in
//! `mw-egress` and must keep the per-hop pin (t22-e7 P4, escalated there).
//!
//! What stays HERE is what is specific to serving remote images to a browser:
//!   * the global concurrency ceiling and the per-account token bucket;
//!   * fetched bytes are re-encoded through the wasm media jail
//!     ([`mw_render::media_jail::reencode_image`], t16-e5) to a metadata-stripped PNG
//!     before serving — a hostile codec never runs natively in this process;
//!   * re-encoded bytes are cached in memory under `(egress route, URL)`, bounded by
//!     **bytes** and by a TTL, and served with a content-hash `ETag` ([`ProxyCache`]);
//!   * the request originates upstream with a normalized `User-Agent` and no forwarded
//!     `Cookie`/`Referer`/`Authorization` (nothing from the browser is proxied).
//!
//! The proxy REQUIRES a session ([`crate::authed`]) so it is never an open relay.
//!
//! # The grants are enforced HERE, not only in the client (t22-e7 P3)
//! The four grant scopes gate what the client rewrites AND what this server will
//! fetch. Until 26.20 only the former was true: `proxy_image` required a session and
//! then fetched any URL, so any authenticated session was an anonymizing fetch relay
//! for arbitrary public URLs — rate-limited, but not scoped to a message the reader
//! had actually consented to load images for.
//!
//! Enforcing it needed the wire to carry what the scopes are keyed on. Every scope in
//! [`mw_store::Store::remote_image_allowed`] is MESSAGE context — `single` is the
//! message id, `per-sender` and `per-domain` come from that message's SENDER — and an
//! image URL's host (`cdn.example`) has no relation to a sender's domain, so nothing
//! in a bare `?url=` request could be resolved into any of them. `t22-e4` added
//! `&emailId=` (`apps/web/src/api/remote-images.ts::imageProxyUrl`, threaded from
//! `Reader.tsx`); [`grant_covers`] resolves it here.
//!
//! Deny-by-default, and the order is load-bearing — see [`grant_covers`] for the
//! decision and [`ungranted_response`] for what a refusal costs and reveals. The gate
//! sits ahead of the cache, so an ungranted session cannot read images a granted one
//! fetched.
//!
//! What it does NOT do: the scopes are per-account and per-message, not per-URL, so a
//! session holding a covering grant may still proxy any *public* URL it names under
//! that message's id. Narrowing the fetch to URLs that actually occur in the message
//! body would need the body at fetch time and is not attempted here.
//!
//! # Ownership
//! This module exposes [`image_proxy_router`]; `crate::lib` (t16-e10, chain link 3)
//! MOUNTS it and applies [`SHELL_CSP_TIGHTENED`] at the shell-CSP site — this module
//! does not edit `lib.rs`. It also re-exports the three `mw-egress` items in-tree
//! callers already reach for through this path ([`ip_allowed`], [`embedded_ipv4s`],
//! [`fetch_url_hardened`]), so `sieve_sync.rs` and `import_routes.rs` are unchanged
//! by the extraction.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use mw_egress::{Refusal, fetch_remote};

use crate::AppState;

/// The egress policy, re-exported at the path in-tree callers already use:
/// `sieve_sync.rs` builds its deliberately NARROWER ManageSieve policy on
/// [`ip_allowed`] + [`embedded_ipv4s`], and `import_routes.rs` fetches `webcal://`
/// subscriptions through [`fetch_url_hardened`]. Re-exporting rather than editing
/// those call sites keeps the 26.20 extraction a move.
pub(crate) use mw_egress::{embedded_ipv4s, fetch_url_hardened, ip_allowed};

// ── S10: tightened shell CSP (delivered here; applied at lib.rs:102 by e10) ──────

/// The tightened Content-Security-Policy for the SPA shell (SPEC §7.4, t16 S10).
/// Delivered as a constant so t16-e10 applies it in `lib.rs` with a one-line change,
/// keeping this milestone's single `lib.rs` editor on the chain.
///
/// Two changes vs the prior shell CSP:
///   * **`require-trusted-types-for 'script'`** — DOM-XSS sink injection must go
///     through a Trusted Types policy. No `trusted-types` allow-list directive is
///     added, so the SPA may keep naming its own policy; only the enforcement is
///     turned on.
///   * **`style-src 'self'`** — the `'unsafe-inline'` style source is dropped.
///
/// NOTE for e10 / e-e2e: dropping style `'unsafe-inline'` blocks inline `style="…"`
/// attributes the SPA framework may emit. If the shell renders broken under this
/// value, the minimal fallback that still satisfies S10's intent is to re-admit
/// inline styles for the *attribute* sink ONLY (`style-src-elem 'self'; style-src-attr
/// 'unsafe-inline'`) rather than restoring the blanket `'unsafe-inline'`. Verify in
/// the live web gate before release. The per-message body CSP (`MESSAGE_CSP`) is
/// unaffected by this constant.
pub(crate) const SHELL_CSP_TIGHTENED: &str = "default-src 'none'; \
     script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; \
     img-src 'self' blob: data:; font-src 'self'; connect-src 'self' blob:; \
     frame-src 'self'; worker-src 'self' blob:; base-uri 'none'; form-action 'none'; \
     require-trusted-types-for 'script'";

// ── fetch caps ───────────────────────────────────────────────────────────────
//
// The size cap, the per-hop timeout and the redirect-hop ceiling moved to
// `mw_egress` with the fetch they bound (`MAX_IMAGE_BYTES`, `FETCH_TIMEOUT`,
// `MAX_REDIRECTS`). The two below are the image proxy's own.

/// Global concurrent-fetch ceiling — bounds proxy load + upstream fan-out.
const MAX_CONCURRENT: usize = 16;

// ── re-encoded-image cache bounds (t22-e7 P2) ─────────────────────────────────
//
// An entry count alone does not bound memory. `MAX_IMAGE_BYTES` caps the bytes we
// FETCH; the cached value is the re-encoded PNG, whose size is a function of the
// decoded image, not of the source — a small, heavily-compressed JPEG can re-encode
// into a far larger lossless PNG. 256 entries with no byte ceiling is therefore an
// unbounded resident set in practice. The bounds below are all enforced together in
// [`ProxyCache::put`]; the byte budget is the one that binds first on realistic
// images, and the entry count survives as a cheap secondary cap.

/// In-memory re-encoded-image cache capacity (entries) before FIFO eviction.
const CACHE_CAPACITY: usize = 256;
/// Total re-encoded bytes the cache may hold. FIFO eviction runs until an insert
/// fits — this is the ceiling on the proxy's resident image memory.
const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;
/// The largest single re-encode worth a cache slot. A bigger one is still SERVED,
/// just not retained: admitting it would evict a large share of the cache for one
/// rarely-repeated image, and an entry above [`MAX_CACHE_BYTES`] could never fit at
/// all.
const MAX_ENTRY_BYTES: usize = 4 * 1024 * 1024;
/// How long a cached re-encode stays servable. Bounds both staleness (the upstream
/// bytes behind a URL can change) and retention of an image no one is reading any
/// more; the `Cache-Control` we hand the browser is separate and unaffected.
const CACHE_TTL: Duration = Duration::from_secs(3600);

/// The egress route the bytes were fetched over — the first half of the cache key.
///
/// Today exactly one route exists (a direct, pinned connection), so this constant is
/// the only value in play and the key behaves like the URL-only key it replaces. It
/// is in the key because the cache is GLOBAL across accounts: the moment fetches can
/// take different egress paths, a URL-only key lets bytes fetched over one path be
/// served to a request that was supposed to take another. That is a property to
/// build in while the route is constant, not after it stops being one. `t22-e14`
/// (wave 4) selects the route per fetch and substitutes its id here.
const DIRECT_ROUTE: &str = "direct";

// ── router ───────────────────────────────────────────────────────────────────

/// The image-proxy + remote-image-grant routes (mounted by t16-e10). Every route is
/// session-authed; the proxy fetch is additionally SSRF-gated.
pub(crate) fn image_proxy_router() -> Router<AppState> {
    Router::new()
        .route("/api/image-proxy", get(proxy_image))
        .route("/api/remote-images/grants", get(list_grants))
        .route("/api/remote-images/grant", post(grant))
        .route("/api/remote-images/revoke", post(revoke))
}

// ── SSRF refusal → HTTP ────────────────────────────────────────────────────────

/// Map an [`mw_egress::Refusal`] onto the wire response, unchanged from when the
/// enum lived here and carried its own `IntoResponse`: the same status for the same
/// discriminant and the same body text. It is a free function rather than a trait
/// impl because both the type and `IntoResponse` are now foreign to this crate; the
/// HTTP mapping is this module's concern in either case, since `mw-egress` serves no
/// HTTP and must not link axum.
///
/// Every arm maps to a client error or bad-gateway — an image request never reveals
/// internal reachability beyond a coarse status + reason.
fn refusal_response(refusal: Refusal) -> Response {
    let (code, msg) = match refusal {
        Refusal::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
        Refusal::Blocked => (StatusCode::FORBIDDEN, "target address is not permitted"),
        Refusal::Timeout => (StatusCode::GATEWAY_TIMEOUT, "upstream timed out"),
        Refusal::Upstream => (StatusCode::BAD_GATEWAY, "upstream fetch failed"),
        Refusal::TooLarge => (StatusCode::BAD_GATEWAY, "upstream image too large"),
    };
    (code, msg).into_response()
}

// ── content-hash cache ─────────────────────────────────────────────────────────

struct CacheEntry {
    etag: String,
    png: Vec<u8>,
    /// When the entry was written, for the [`CACHE_TTL`] check.
    inserted: Instant,
}

/// What a cached re-encode is filed under: the egress route that produced the bytes
/// ([`DIRECT_ROUTE`] today) and the requested URL. Never the account — the cache is
/// deliberately shared, since the bytes are a public resource fetched with nothing of
/// the reader's attached, and a per-account cache would multiply the resident set by
/// the account count for identical images.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct CacheKey {
    route: String,
    url: String,
}

impl CacheKey {
    fn new(route: &str, url: &str) -> Self {
        Self {
            route: route.to_string(),
            url: url.to_string(),
        }
    }
}

/// A bounded FIFO cache of re-encoded images. The `ETag` is the content hash of the
/// re-encoded PNG, so a repeat load is served from memory and the browser can
/// revalidate cheaply.
///
/// Three bounds, all enforced on insert: [`CACHE_TTL`] (age), [`MAX_CACHE_BYTES`]
/// (total retained bytes, the one that binds first) and [`CACHE_CAPACITY`] (entries).
/// `bytes` is maintained as the exact sum of the retained `png` lengths so the byte
/// budget costs no traversal.
struct ProxyCache {
    map: HashMap<CacheKey, CacheEntry>,
    order: VecDeque<CacheKey>,
    bytes: usize,
}

impl ProxyCache {
    /// A FRESH entry's `(etag, png)`, or `None`. An entry past its TTL is dropped
    /// here rather than returned, so a stale hit never shortcuts the re-fetch.
    fn get(&mut self, key: &CacheKey) -> Option<(String, Vec<u8>)> {
        match self.map.get(key) {
            Some(e) if e.inserted.elapsed() < CACHE_TTL => {}
            Some(_) => {
                self.remove(key);
                return None;
            }
            None => return None,
        }
        self.map.get(key).map(|e| (e.etag.clone(), e.png.clone()))
    }

    /// Drop one entry, keeping `bytes` and `order` consistent with `map`.
    fn remove(&mut self, key: &CacheKey) {
        if let Some(e) = self.map.remove(key) {
            self.bytes = self.bytes.saturating_sub(e.png.len());
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
            }
        }
    }

    /// Drop every entry past [`CACHE_TTL`]. Called on insert so an idle-then-busy
    /// proxy does not carry an hour-old resident set into its next burst.
    fn evict_expired(&mut self) {
        let stale: Vec<CacheKey> = self
            .map
            .iter()
            .filter(|(_, e)| e.inserted.elapsed() >= CACHE_TTL)
            .map(|(k, _)| k.clone())
            .collect();
        for k in stale {
            self.remove(&k);
        }
    }

    /// Insert (or refresh) an entry, then evict until every bound holds.
    fn put(&mut self, key: CacheKey, etag: String, png: Vec<u8>) {
        // Too large to retain — serve it, forget it. Also what guarantees the
        // eviction loop below terminates with the entry admitted.
        if png.len() > MAX_ENTRY_BYTES {
            self.remove(&key);
            return;
        }
        // NOT first-writer-wins: the previous value for this key is dropped and
        // replaced. A URL whose entry has just expired, or whose upstream bytes
        // changed, must be able to take its slot back — under the old early-return an
        // entry could never be refreshed, only evicted by unrelated traffic.
        self.remove(&key);
        self.evict_expired();
        while !self.order.is_empty()
            && (self.map.len() >= CACHE_CAPACITY || self.bytes + png.len() > MAX_CACHE_BYTES)
        {
            let Some(old) = self.order.pop_front() else {
                break;
            };
            if let Some(e) = self.map.remove(&old) {
                self.bytes = self.bytes.saturating_sub(e.png.len());
            }
        }
        self.bytes += png.len();
        self.order.push_back(key.clone());
        self.map.insert(
            key,
            CacheEntry {
                etag,
                png,
                inserted: Instant::now(),
            },
        );
    }
}

fn cache() -> &'static Mutex<ProxyCache> {
    static CACHE: OnceLock<Mutex<ProxyCache>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Mutex::new(ProxyCache {
            map: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
        })
    })
}

/// The global concurrent-fetch limiter.
fn fetch_semaphore() -> &'static tokio::sync::Semaphore {
    static SEM: OnceLock<tokio::sync::Semaphore> = OnceLock::new();
    SEM.get_or_init(|| tokio::sync::Semaphore::new(MAX_CONCURRENT))
}

// ── per-account rate limit (R6, t18) ────────────────────────────────────────────

/// Sustained refill rate: tokens added per second per account (≈60 fetches/min).
const RATE_REFILL_PER_SEC: f64 = 1.0;
/// Burst ceiling: the most fetches an idle account can spend at once.
const RATE_BURST: f64 = 120.0;

/// A single account's token bucket.
struct TokenBucket {
    tokens: f64,
    last: std::time::Instant,
}

/// A coarse in-memory per-account token-bucket limiter for the image-proxy FETCH
/// path (R6). It caps how fast one account can drive DISTINCT upstream fetches
/// (cache hits are free — see [`proxy_image`]), a fan-out/abuse limit rather than a
/// security boundary (the SSRF gate is that).
///
/// Caveat (documented, by design): state lives in a process-local `OnceLock` static,
/// so it resets on restart and is **per-replica** — N replicas each admit the full
/// rate. A cluster-global limit would need a shared store and a hot-path write, not
/// warranted for an abuse cap. The account map is bounded by the deployment's account
/// count (one small bucket per account that has used the proxy); no eviction needed.
struct AccountRateLimiter {
    buckets: HashMap<String, TokenBucket>,
}

impl AccountRateLimiter {
    /// Charge one token to `account_id`, refilling for elapsed time first. Returns
    /// `true` if a token was available (request allowed), `false` if the bucket is
    /// exhausted (→ `429`). A never-seen account starts with a full burst.
    fn check(&mut self, account_id: &str) -> bool {
        let now = std::time::Instant::now();
        let b = self
            .buckets
            .entry(account_id.to_string())
            .or_insert(TokenBucket {
                tokens: RATE_BURST,
                last: now,
            });
        let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
        b.tokens = (b.tokens + elapsed * RATE_REFILL_PER_SEC).min(RATE_BURST);
        b.last = now;
        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

fn rate_limiter() -> &'static Mutex<AccountRateLimiter> {
    static RL: OnceLock<Mutex<AccountRateLimiter>> = OnceLock::new();
    RL.get_or_init(|| {
        Mutex::new(AccountRateLimiter {
            buckets: HashMap::new(),
        })
    })
}

/// Quoted-hex `ETag` of the re-encoded bytes.
fn etag_for(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(2 + digest.len() * 2 + 1);
    s.push('"');
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s.push('"');
    s
}

// ── handlers ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProxyQuery {
    url: String,
    /// The message this image is being loaded FOR — what every grant scope is keyed
    /// on. Optional on the wire (a request may simply omit it) but NOT optional for
    /// the fetch: without it no scope can match, so [`grant_covers`] refuses.
    #[serde(default)]
    email_id: Option<String>,
}

/// Whether `account_id` holds a grant covering the message `email_id` — the
/// server-side half of the remote-image grant model (t22-e7 P3).
///
/// Deny-by-default at every step: no id, an id that does not resolve, an id
/// belonging to ANOTHER account, or no covering grant all return `false`. The
/// account check matters because grants are per-account — an id borrowed from
/// another account must not unlock that account's sender scopes.
///
/// All four scopes are resolved by [`mw_store::Store::remote_image_allowed`] in one
/// query rather than reassembled here, so this cannot drift from the model the grant
/// endpoints below write.
///
/// `Err(())` is a store failure, distinct from "not granted", so a database problem
/// surfaces as a `500` instead of silently reading as a denied grant.
async fn grant_covers(
    store: &mw_store::Store,
    account_id: &str,
    email_id: Option<&str>,
) -> Result<bool, ()> {
    let Some(email_id) = email_id.filter(|s| !s.is_empty()) else {
        return Ok(false);
    };
    let msg = match store.get_message(email_id).await {
        Ok(m) => m,
        Err(mw_store::StoreError::NotFound) => return Ok(false),
        Err(_) => return Err(()),
    };
    if msg.account_id != account_id {
        return Ok(false);
    }
    let (sender, domain) = sender_of(store, email_id).await;
    store
        .remote_image_allowed(account_id, email_id, &sender, &domain)
        .await
        .map_err(|_| ())
}

/// A message's sender address and its domain, both lower-cased.
///
/// Derived exactly as the client derives the values it GRANTS, or the two would
/// never match: the first `from` address (`Reader.tsx`'s
/// `props.email.from?.[0]?.email ?? ''`) and the part after its last `@`
/// (`remote-images.ts::senderDomain`).
///
/// Empty when the message has no stored envelope or no `from`. Such a message can
/// then only be covered by an `all` or `single` grant — which is the honest answer,
/// since nothing is known about its sender.
async fn sender_of(store: &mw_store::Store, email_id: &str) -> (String, String) {
    let Ok(Some(bytes)) = store.get_envelope(email_id).await else {
        return (String::new(), String::new());
    };
    let Ok(email) = serde_json::from_slice::<mw_jmap::Email>(&bytes) else {
        return (String::new(), String::new());
    };
    let sender = email
        .from
        .as_ref()
        .and_then(|v| v.first())
        .map(|a| a.email.trim().to_lowercase())
        .unwrap_or_default();
    let domain = match sender.rfind('@') {
        Some(at) => sender[at + 1..].to_string(),
        None => String::new(),
    };
    (sender, domain)
}

/// `GET /api/image-proxy?url=…` — session-authed, SSRF-gated fetch → wasm-jail
/// re-encode → PNG. Served same-origin so the shell's `img-src 'self'` covers it.
async fn proxy_image(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ProxyQuery>,
) -> Response {
    // Require a session — never an open relay. Capture it for the per-account rate
    // limit below.
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };

    // P3: the grant gate comes BEFORE the cache, or a session with no grant could
    // read images another session fetched.
    match grant_covers(&state.store, &session.account_id, q.email_id.as_deref()).await {
        Ok(true) => {}
        Ok(false) => return ungranted_response(&session.account_id, &q.url).await,
        Err(()) => return internal("remote-image grant check"),
    }

    // Serve a cache hit before doing any work (and honor If-None-Match). A cache hit
    // performs no upstream fetch, so it does NOT consume the per-account rate budget.
    let key = CacheKey::new(DIRECT_ROUTE, &q.url);
    if let Some((etag, png)) = cache().lock().expect("image cache lock").get(&key) {
        if if_none_match(&headers, &etag) {
            return not_modified(&etag);
        }
        return image_response(png, etag);
    }

    // Per-account fan-out rate limit (R6): a cache MISS will fetch upstream, so
    // charge the account one token here. Exhaustion → 429 (per-replica; see
    // `AccountRateLimiter`).
    if !rate_limiter()
        .lock()
        .expect("image rate-limit lock")
        .check(&session.account_id)
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "image proxy rate limit exceeded",
        )
            .into_response();
    }

    let url = match reqwest::Url::parse(&q.url) {
        Ok(u) => u,
        Err(_) => return refusal_response(Refusal::BadRequest("malformed URL")),
    };

    // Bound concurrency; shed load rather than queue unboundedly.
    let _permit = match fetch_semaphore().try_acquire() {
        Ok(p) => p,
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "image proxy busy").into_response();
        }
    };

    let raw = match fetch_remote(url).await {
        Ok(b) => b,
        Err(r) => return refusal_response(r),
    };

    // Re-encode in the wasm media jail. `reencode_image` is CPU-bound + blocking
    // (a bounded wasmtime interpreter run), so run it off the async runtime.
    let png = match tokio::task::spawn_blocking(move || mw_render::media_jail::reencode_image(&raw))
        .await
    {
        Ok(Ok(png)) => png,
        // A decode/re-encode failure means the bytes were not a usable image (or a
        // hostile codec tripped the jail) — refuse rather than serve them.
        Ok(Err(_)) => {
            return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "not a decodable image").into_response();
        }
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "re-encode failed").into_response(),
    };

    let etag = etag_for(&png);
    cache()
        .lock()
        .expect("image cache lock")
        .put(key, etag.clone(), png.clone());
    image_response(png, etag)
}

/// The response for a request no grant covers.
///
/// It charges the rate limiter and then runs the egress policy, in that order, and
/// only says "not granted" if the URL would otherwise have been fetchable.
///
/// Both steps are deliberate. Charging keeps the fan-out cap meaningful on exactly
/// the requests most worth capping — a session with no grant retrying in a loop —
/// and keeps the limiter's existing behaviour, where a refused request is charged and
/// only a cache hit is free. Running the policy first keeps a refusal reported as
/// what it is: a `file://` URL stays a `400` and a loopback target stays a `403`
/// whether or not the caller holds a grant, so the grant gate never becomes a way to
/// tell a granted session's refusals apart from an ungranted one's. Neither step
/// fetches anything, and the policy runs exactly once on this path — a granted
/// request runs it inside `fetch_remote` instead, never twice.
///
/// Known residual, stated rather than implied: an authenticated session with no grant
/// can still cause a DNS resolution of an arbitrary host here, and can still learn
/// from the status code whether that host resolves to a blocked address. That is not
/// new — before the gate the same session could resolve AND fetch it — but the gate
/// does not close it.
async fn ungranted_response(account_id: &str, raw_url: &str) -> Response {
    if !rate_limiter()
        .lock()
        .expect("image rate-limit lock")
        .check(account_id)
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "image proxy rate limit exceeded",
        )
            .into_response();
    }
    let url = match reqwest::Url::parse(raw_url) {
        Ok(u) => u,
        Err(_) => return refusal_response(Refusal::BadRequest("malformed URL")),
    };
    if let Err(r) = mw_egress::validate_and_resolve(url).await {
        return refusal_response(r);
    }
    (
        StatusCode::FORBIDDEN,
        "no remote-image grant covers this message",
    )
        .into_response()
}

/// Build a `200` image response with the content-hash `ETag` + private caching.
fn image_response(png: Vec<u8>, etag: String) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png".to_string()),
            (header::CACHE_CONTROL, "private, max-age=86400".to_string()),
            (header::ETAG, etag),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
        ],
        png,
    )
        .into_response()
}

fn not_modified(etag: &str) -> Response {
    (StatusCode::NOT_MODIFIED, [(header::ETAG, etag.to_string())]).into_response()
}

/// Whether the request's `If-None-Match` covers `etag`.
fn if_none_match(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|inm| inm == "*" || inm.split(',').any(|t| t.trim() == etag))
        .unwrap_or(false)
}

// ── grant endpoints (S8, over the 0016 4-scope model) ──────────────────────────

/// One of the four grant scopes; anything else is refused.
fn valid_scope_kind(kind: &str) -> bool {
    matches!(kind, "single" | "all" | "per-sender" | "per-domain")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrantReq {
    scope_kind: String,
    #[serde(default)]
    scope_value: String,
}

/// `GET /api/remote-images/grants` — the caller's active (non-revoked) grants.
async fn list_grants(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match state
        .store
        .list_active_image_grants(&session.account_id)
        .await
    {
        Ok(rows) => {
            let list: Vec<_> = rows
                .iter()
                .map(|g| {
                    json!({
                        "scopeKind": g.scope_kind,
                        "scopeValue": g.scope_value,
                        "grantedAt": g.granted_at,
                    })
                })
                .collect();
            Json(json!({ "accountId": session.account_id, "list": list })).into_response()
        }
        Err(_) => internal("list grants"),
    }
}

/// `POST /api/remote-images/grant` — grant remote-image loading for a scope
/// (idempotent; un-revokes). The scope is applied to the CALLER's account; a
/// client-supplied account id is never trusted.
async fn grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<GrantReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if !valid_scope_kind(&body.scope_kind) {
        return (StatusCode::BAD_REQUEST, "unknown grant scope").into_response();
    }
    // `all` is account-wide: pin its value to "" so it cannot masquerade as a
    // narrower scope.
    let value = if body.scope_kind == "all" {
        ""
    } else {
        body.scope_value.trim()
    };
    match state
        .store
        .grant_remote_image(&session.account_id, &body.scope_kind, value)
        .await
    {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => internal("grant remote image"),
    }
}

/// `POST /api/remote-images/revoke` — soft-revoke a grant (blocks again next load).
async fn revoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<GrantReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if !valid_scope_kind(&body.scope_kind) {
        return (StatusCode::BAD_REQUEST, "unknown grant scope").into_response();
    }
    let value = if body.scope_kind == "all" {
        ""
    } else {
        body.scope_value.trim()
    };
    match state
        .store
        .revoke_remote_image(&session.account_id, &body.scope_kind, value)
        .await
    {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => internal("revoke remote image"),
    }
}

fn internal(ctx: &str) -> Response {
    tracing::error!("image proxy: {ctx} failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::time::Duration;

    use mw_egress::{Hop, Target, fetch_hop};

    // ── a LOCAL origin for the re-encode leg ──────────────────────────────────
    //
    // The SSRF gate, the URL/scheme checks and the fetch mechanics (size cap,
    // timeout, redirect surfacing, and the `no_proxy` assertion) moved to
    // `mw-egress` with the code they cover and are tested there — including
    // through the crate's public API in `mw-egress/tests/policy_through_crate.rs`.
    //
    // What remains here is the leg that is this module's own: fetched bytes →
    // wasm media jail → stripped PNG. It calls the low-level `fetch_hop` with a
    // pinned loopback address DIRECTLY, deliberately bypassing the gate (which —
    // correctly — would refuse the 127.0.0.1 test server).

    async fn spawn_origin(
        body: Vec<u8>,
        delay: Option<Duration>,
        status: StatusCode,
        location: Option<String>,
    ) -> SocketAddr {
        use axum::routing::get as aget;
        let handler = move || async move {
            if let Some(d) = delay {
                tokio::time::sleep(d).await;
            }
            let mut resp = Response::new(axum::body::Body::from(body));
            *resp.status_mut() = status;
            if let Some(loc) = location {
                resp.headers_mut()
                    .insert(header::LOCATION, loc.parse().unwrap());
            }
            resp
        };
        let app: Router = Router::new().route("/img", aget(handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        addr
    }

    fn target_for(addr: SocketAddr) -> Target {
        Target {
            url: reqwest::Url::parse(&format!("http://{addr}/img")).unwrap(),
            host: addr.ip().to_string(),
            addr,
        }
    }

    // ── re-encode integration: fetched bytes → wasm jail → PNG ────────────────

    #[tokio::test]
    async fn fetched_image_reencodes_to_stripped_png() {
        // 1×1 GIF served by a local origin; fetched (gate-bypassed) then re-encoded
        // in the media jail → a normalized PNG (metadata stripped by the jail).
        let gif: Vec<u8> = vec![
            0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x21, 0xF9, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2C,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00,
            0x3B,
        ];
        let addr = spawn_origin(gif, None, StatusCode::OK, None).await;
        let bytes = match fetch_hop(&target_for(addr), "image/*").await.unwrap() {
            Hop::Body(b) => b,
            Hop::Redirect(_) => panic!("unexpected redirect"),
        };
        let png = mw_render::media_jail::reencode_image(&bytes).expect("re-encode");
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    // ── cache + etag ───────────────────────────────────────────────────────────

    fn empty_cache() -> ProxyCache {
        ProxyCache {
            map: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
        }
    }

    /// A key on the one route that exists today, for the tests that are not about
    /// routing.
    fn k(url: &str) -> CacheKey {
        CacheKey::new(DIRECT_ROUTE, url)
    }

    #[test]
    fn cache_is_bounded_fifo() {
        let mut c = empty_cache();
        for i in 0..(CACHE_CAPACITY + 10) {
            c.put(k(&format!("k{i}")), format!("\"{i}\""), vec![i as u8]);
        }
        assert!(c.map.len() <= CACHE_CAPACITY);
        // The earliest keys were evicted.
        assert!(c.get(&k("k0")).is_none());
        assert!(c.get(&k(&format!("k{}", CACHE_CAPACITY + 9))).is_some());
    }

    // ── P2: the cache is bounded by BYTES, not only by entry count ──────────────

    #[test]
    fn cache_byte_budget_evicts_while_far_under_the_entry_cap() {
        // The corpus is chosen so ONLY the byte bound can fire: entries of the
        // largest cacheable size, enough of them to exceed MAX_CACHE_BYTES, and few
        // enough that CACHE_CAPACITY is never approached. If eviction happens here it
        // happened on bytes.
        let n = MAX_CACHE_BYTES / MAX_ENTRY_BYTES + 2; // 10 entries at 4 MiB = 40 MiB
        assert!(
            n < CACHE_CAPACITY,
            "the corpus must stay under the entry cap ({n} vs {CACHE_CAPACITY}) or this \
             test cannot tell which bound fired"
        );
        assert!(
            n * MAX_ENTRY_BYTES > MAX_CACHE_BYTES,
            "the corpus must exceed the byte budget"
        );

        let mut c = empty_cache();
        for i in 0..n {
            c.put(
                k(&format!("big{i}")),
                format!("\"{i}\""),
                vec![7u8; MAX_ENTRY_BYTES],
            );
        }

        // The entry cap was never in play, yet entries were dropped.
        assert!(
            c.map.len() < n,
            "the byte budget must have evicted (kept all {n})"
        );
        assert!(c.map.len() < CACHE_CAPACITY);
        assert!(
            c.bytes <= MAX_CACHE_BYTES,
            "retained {} bytes over a {MAX_CACHE_BYTES}-byte budget",
            c.bytes
        );
        // The accounting is exact, not approximate — `bytes` equals what is held.
        let held: usize = c.map.values().map(|e| e.png.len()).sum();
        assert_eq!(
            c.bytes, held,
            "byte counter drifted from the retained entries"
        );
        // FIFO: the oldest went, the newest stayed.
        assert!(c.get(&k("big0")).is_none());
        assert!(c.get(&k(&format!("big{}", n - 1))).is_some());
    }

    #[test]
    fn an_oversized_reencode_is_not_cached_and_evicts_nothing() {
        let mut c = empty_cache();
        c.put(k("small"), "\"s\"".into(), vec![1u8; 1024]);
        // One byte over the per-entry cap: refused a slot, and the existing entry
        // survives (an oversized image must not be able to flush the cache).
        c.put(k("huge"), "\"h\"".into(), vec![2u8; MAX_ENTRY_BYTES + 1]);
        assert!(
            c.get(&k("huge")).is_none(),
            "oversized entry must not be retained"
        );
        assert!(
            c.get(&k("small")).is_some(),
            "oversized insert must not evict"
        );
        assert_eq!(c.bytes, 1024);
    }

    // ── P2: TTL ────────────────────────────────────────────────────────────────

    #[test]
    fn a_cache_entry_expires_after_the_ttl() {
        let mut c = empty_cache();
        c.put(k("u"), "\"e\"".into(), vec![9u8; 32]);
        assert!(c.get(&k("u")).is_some(), "fresh entry serves");

        // Age it past the TTL by moving its insert time backwards.
        let entry = c.map.get_mut(&k("u")).unwrap();
        entry.inserted = entry
            .inserted
            .checked_sub(CACHE_TTL + Duration::from_secs(1))
            .expect("shift insert time back");

        assert!(
            c.get(&k("u")).is_none(),
            "an entry past its TTL must not serve"
        );
        // ...and the expired bytes are released, not merely hidden.
        assert_eq!(
            c.bytes, 0,
            "expired entry must be dropped, not just skipped"
        );
        assert!(c.order.is_empty());
    }

    #[test]
    fn put_refreshes_an_existing_key_rather_than_first_writer_wins() {
        let mut c = empty_cache();
        c.put(k("u"), "\"v1\"".into(), vec![1u8; 10]);
        c.put(k("u"), "\"v2\"".into(), vec![2u8; 20]);
        let (etag, png) = c.get(&k("u")).expect("entry present");
        assert_eq!(etag, "\"v2\"", "the later write must win");
        assert_eq!(png, vec![2u8; 20]);
        // Exactly one entry, and the byte count reflects the replacement only.
        assert_eq!(c.map.len(), 1);
        assert_eq!(
            c.order.len(),
            1,
            "replacing must not leave a stale order slot"
        );
        assert_eq!(c.bytes, 20, "the replaced entry's bytes must be released");
    }

    // ── P2: the key is (route, url), not url ───────────────────────────────────

    #[test]
    fn cache_is_keyed_by_route_and_url_not_url_alone() {
        let mut c = empty_cache();
        let same_url = "https://cdn.example/logo.png";
        c.put(
            CacheKey::new(DIRECT_ROUTE, same_url),
            "\"d\"".into(),
            vec![1u8; 8],
        );
        c.put(
            CacheKey::new("via-proxy-1", same_url),
            "\"p\"".into(),
            vec![2u8; 8],
        );

        // One URL, two routes, two entries — bytes fetched over one egress path are
        // never served to a request that took another.
        assert_eq!(c.map.len(), 2);
        assert_eq!(
            c.get(&CacheKey::new(DIRECT_ROUTE, same_url)).unwrap().0,
            "\"d\""
        );
        assert_eq!(
            c.get(&CacheKey::new("via-proxy-1", same_url)).unwrap().0,
            "\"p\""
        );
    }

    // ── R6: per-account token-bucket rate limit ──────────────────────────────────

    #[test]
    fn rate_limiter_allows_burst_then_429s_and_is_per_account() {
        let mut rl = AccountRateLimiter {
            buckets: HashMap::new(),
        };
        // A fresh account spends its full burst, then the next request is refused
        // (no measurable time elapses inside the loop → no refill).
        for i in 0..RATE_BURST as usize {
            assert!(rl.check("acct-a"), "burst token {i} should be admitted");
        }
        assert!(
            !rl.check("acct-a"),
            "exhausted bucket must return false (429)"
        );
        // A different account has an independent budget.
        assert!(
            rl.check("acct-b"),
            "a second account is limited independently"
        );
    }

    #[test]
    fn rate_limiter_refills_over_time() {
        let mut rl = AccountRateLimiter {
            buckets: HashMap::new(),
        };
        for _ in 0..RATE_BURST as usize {
            assert!(rl.check("a"));
        }
        assert!(!rl.check("a"), "bucket drained");
        // Simulate ~2 seconds elapsed: at 1 token/s that is ~2 refilled tokens.
        if let Some(b) = rl.buckets.get_mut("a") {
            b.last = b
                .last
                .checked_sub(std::time::Duration::from_secs(2))
                .unwrap_or(b.last);
        }
        assert!(rl.check("a"), "first refilled token available");
        assert!(rl.check("a"), "second refilled token available");
        assert!(!rl.check("a"), "only ~2 tokens refilled, third is refused");
    }

    #[test]
    fn etag_is_stable_content_hash() {
        assert_eq!(etag_for(b"abc"), etag_for(b"abc"));
        assert_ne!(etag_for(b"abc"), etag_for(b"abd"));
        assert!(etag_for(b"abc").starts_with('"'));
    }

    #[test]
    fn csp_tightens_style_and_adds_trusted_types() {
        assert!(SHELL_CSP_TIGHTENED.contains("require-trusted-types-for 'script'"));
        assert!(SHELL_CSP_TIGHTENED.contains("style-src 'self';"));
        assert!(!SHELL_CSP_TIGHTENED.contains("style-src 'self' 'unsafe-inline'"));
    }

    // ── P3: server-side grant enforcement, BOTH directions ─────────────────────
    //
    // These drive `grant_covers` against a real store, which is where the decision
    // is made; the handler's only job is to call it before the cache and the fetch.
    // A refusal-only test would pass with the feature deleted, so every case below
    // asserts the pair: the same request refused without a grant and admitted with
    // one.

    use mw_store::{
        AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    };

    /// A real store with a real account + mailbox, so messages carry the account id
    /// the FK and the grant check both read.
    struct Fixture {
        store: Store,
        account: String,
        mailbox: String,
    }

    async fn fixture() -> Fixture {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let (account, mailbox) = seed_account(&store, "reader@example.org", 1).await;
        Fixture {
            store,
            account,
            mailbox,
        }
    }

    async fn seed_account(store: &Store, username: &str, uidvalidity: u32) -> (String, String) {
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "imap.example.org",
                    port: 993,
                    tls: "implicit",
                    username,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: username.to_string(),
                    password: "pw".into(),
                },
            )
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity,
                uidnext: 1,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        (account, mailbox)
    }

    /// Store a message and return the stable id the store assigned it.
    async fn seed_message(
        store: &Store,
        account: &str,
        mailbox: &str,
        uid: u32,
        from: Option<&str>,
    ) -> String {
        let envelope = from.map(|addr| {
            serde_json::to_vec(&json!({ "from": [{ "email": addr }] })).expect("envelope")
        });
        store
            .upsert_message(&MessageUpsert {
                account_id: account,
                mailbox_id: mailbox,
                uid,
                uidvalidity: 1,
                message_id: None,
                thread_id: None,
                internaldate: None,
                size: 10,
                flags_json: "[]",
                envelope: envelope.as_deref(),
                blob_ref: None,
            })
            .await
            .expect("seed message")
    }

    impl Fixture {
        async fn message(&self, uid: u32, from: Option<&str>) -> String {
            seed_message(&self.store, &self.account, &self.mailbox, uid, from).await
        }
    }

    #[tokio::test]
    async fn an_account_without_a_grant_is_refused_and_with_one_is_admitted() {
        let f = fixture().await;
        let m = f.message(1, Some("sales@shop.example")).await;

        // Deny-by-default: the message exists and belongs to the caller, but nothing
        // is granted.
        assert!(
            !grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
            "no grant must refuse"
        );

        // The SAME request, once a covering grant exists, is admitted. Without this
        // direction the test would pass with the whole gate deleted.
        f.store
            .grant_remote_image(&f.account, "single", &m)
            .await
            .unwrap();
        assert!(
            grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
            "a covering grant must admit"
        );

        // ...and revoking puts it back to refused.
        f.store
            .revoke_remote_image(&f.account, "single", &m)
            .await
            .unwrap();
        assert!(
            !grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
            "a revoked grant must refuse again"
        );
    }

    #[tokio::test]
    async fn every_one_of_the_four_scopes_admits_and_only_when_it_matches() {
        // Each scope: refused before, admitted after, and NOT admitted for a message
        // the scope does not cover — so no scope can be read as account-wide except
        // `all`, which is the one that means it.
        for kind in ["single", "per-sender", "per-domain", "all"] {
            let f = fixture().await;
            let m = f.message(1, Some("sales@shop.example")).await;
            // A second message from a DIFFERENT sender, same account.
            let other = f.message(2, Some("noreply@other.example")).await;

            let value = match kind {
                "single" => m.clone(),
                "per-sender" => "sales@shop.example".to_string(),
                "per-domain" => "shop.example".to_string(),
                _ => String::new(),
            };

            assert!(!grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
            f.store
                .grant_remote_image(&f.account, kind, &value)
                .await
                .unwrap();
            assert!(
                grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
                "a {kind} grant must cover the message it was granted for"
            );
            assert_eq!(
                grant_covers(&f.store, &f.account, Some(&other))
                    .await
                    .unwrap(),
                kind == "all",
                "{kind} must not leak onto an unrelated message"
            );
        }
    }

    #[tokio::test]
    async fn a_grant_does_not_cross_accounts() {
        let f = fixture().await;
        let m = f.message(1, Some("sales@shop.example")).await;
        let (other_account, _) = seed_account(&f.store, "someone@else.example", 2).await;

        // The other account grants itself everything...
        f.store
            .grant_remote_image(&other_account, "all", "")
            .await
            .unwrap();

        // ...which must not let it proxy for a message it does not own, even though
        // its own grant is as broad as grants get.
        assert!(
            !grant_covers(&f.store, &other_account, Some(&m))
                .await
                .unwrap(),
            "another account's message id must not be usable"
        );
        // ...and must not carry over to the message's real owner, who granted nothing.
        assert!(
            !grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
            "the owner holds no grant of its own"
        );
    }

    #[tokio::test]
    async fn a_missing_or_unknown_message_id_is_refused_even_with_an_account_grant() {
        let f = fixture().await;
        let m = f.message(1, Some("sales@shop.example")).await;
        // The broadest grant there is.
        f.store
            .grant_remote_image(&f.account, "all", "")
            .await
            .unwrap();

        // No id, an empty id, and an id that resolves to nothing all name no message
        // — refused. This is what stops the gate degrading into "the account holds
        // some grant", which would admit any URL for any message.
        assert!(!grant_covers(&f.store, &f.account, None).await.unwrap());
        assert!(!grant_covers(&f.store, &f.account, Some("")).await.unwrap());
        assert!(
            !grant_covers(&f.store, &f.account, Some("no-such-id"))
                .await
                .unwrap()
        );
        // The real id under that same grant IS admitted — the control that proves
        // the refusals above are about the id, not about the grant being missing.
        assert!(grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
    }

    #[tokio::test]
    async fn sender_scopes_are_derived_the_way_the_client_grants_them() {
        // The client grants `sender.toLowerCase()` and `senderDomain(sender)`; the
        // server must derive the same strings from the envelope or the two never
        // match. Mixed case in the envelope, lower-case in the grant.
        let f = fixture().await;
        let m = f.message(1, Some("Sales@Shop.Example")).await;

        let (sender, domain) = sender_of(&f.store, &m).await;
        assert_eq!(sender, "sales@shop.example");
        assert_eq!(domain, "shop.example");

        f.store
            .grant_remote_image(&f.account, "per-sender", "sales@shop.example")
            .await
            .unwrap();
        assert!(grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
    }

    #[tokio::test]
    async fn a_message_with_no_envelope_falls_back_to_all_and_single_only() {
        let f = fixture().await;
        let m = f.message(1, None).await;
        assert_eq!(
            sender_of(&f.store, &m).await,
            (String::new(), String::new())
        );

        // Nothing is known about the sender, so a per-domain grant cannot cover it...
        f.store
            .grant_remote_image(&f.account, "per-domain", "shop.example")
            .await
            .unwrap();
        assert!(!grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
        // ...but a single-message grant still can.
        f.store
            .grant_remote_image(&f.account, "single", &m)
            .await
            .unwrap();
        assert!(grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
    }

    #[test]
    fn only_the_four_scopes_are_valid() {
        for k in ["single", "all", "per-sender", "per-domain"] {
            assert!(valid_scope_kind(k));
        }
        for k in ["", "global", "sender", "domain", "ALL"] {
            assert!(!valid_scope_kind(k));
        }
    }
}
