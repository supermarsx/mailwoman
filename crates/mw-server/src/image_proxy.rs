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
//! What stays HERE is what is specific to serving remote images to a browser:
//!   * the global concurrency ceiling and the per-account token bucket;
//!   * fetched bytes are re-encoded through the wasm media jail
//!     ([`mw_render::media_jail::reencode_image`], t16-e5) to a metadata-stripped PNG
//!     before serving — a hostile codec never runs natively in this process;
//!   * results are cached by content hash (served with an `ETag`);
//!   * the request originates upstream with a normalized `User-Agent` and no forwarded
//!     `Cookie`/`Referer`/`Authorization` (nothing from the browser is proxied).
//!
//! The proxy REQUIRES a session ([`crate::authed`]) so it is never an open relay.
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
/// In-memory re-encoded-image cache capacity (entries) before FIFO eviction.
const CACHE_CAPACITY: usize = 256;

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
}

/// A tiny bounded FIFO cache of re-encoded images, keyed by the requested URL. The
/// `ETag` is the content hash of the re-encoded PNG, so a repeat load is served from
/// memory and the browser can revalidate cheaply.
struct ProxyCache {
    map: HashMap<String, CacheEntry>,
    order: VecDeque<String>,
}

impl ProxyCache {
    fn get(&self, key: &str) -> Option<(String, Vec<u8>)> {
        self.map.get(key).map(|e| (e.etag.clone(), e.png.clone()))
    }
    fn put(&mut self, key: String, etag: String, png: Vec<u8>) {
        if self.map.contains_key(&key) {
            return;
        }
        while self.order.len() >= CACHE_CAPACITY {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            } else {
                break;
            }
        }
        self.order.push_back(key.clone());
        self.map.insert(key, CacheEntry { etag, png });
    }
}

fn cache() -> &'static Mutex<ProxyCache> {
    static CACHE: OnceLock<Mutex<ProxyCache>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Mutex::new(ProxyCache {
            map: HashMap::new(),
            order: VecDeque::new(),
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
struct ProxyQuery {
    url: String,
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

    // Serve a cache hit before doing any work (and honor If-None-Match). A cache hit
    // performs no upstream fetch, so it does NOT consume the per-account rate budget.
    if let Some((etag, png)) = cache().lock().expect("image cache lock").get(&q.url) {
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
        .put(q.url, etag.clone(), png.clone());
    image_response(png, etag)
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

    #[test]
    fn cache_is_bounded_fifo() {
        let mut c = ProxyCache {
            map: HashMap::new(),
            order: VecDeque::new(),
        };
        for i in 0..(CACHE_CAPACITY + 10) {
            c.put(format!("k{i}"), format!("\"{i}\""), vec![i as u8]);
        }
        assert!(c.map.len() <= CACHE_CAPACITY);
        // The earliest keys were evicted.
        assert!(c.get("k0").is_none());
        assert!(c.get(&format!("k{}", CACHE_CAPACITY + 9)).is_some());
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
