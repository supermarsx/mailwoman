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
//! The fetch is deny-by-default egress:
//!   * scheme ∈ {`http`,`https`} only; URLs carrying credentials are refused;
//!   * DNS is resolved **once, by us**, and the fetch is PINNED to the resolved IP
//!     (reqwest `resolve`) so a name cannot rebind to a new address between our check
//!     and the connect (anti-DNS-rebinding);
//!   * every resolved address is checked against [`ip_allowed`] — loopback, private,
//!     link-local (incl. the `169.254.169.254` cloud-metadata address), CGNAT,
//!     unique-local/link-local IPv6, multicast, unspecified and reserved ranges are
//!     REFUSED; IPv4-mapped IPv6 is unwrapped and re-checked;
//!   * redirects are NOT auto-followed — each hop's `Location` is re-parsed and
//!     re-validated through the same gate (a redirect to a private target is refused);
//!   * hard caps bound response size, per-request timeout, and global concurrency;
//!   * the request originates here with a normalized `User-Agent` and no forwarded
//!     `Cookie`/`Referer`/`Authorization` (nothing from the browser is proxied);
//!   * fetched bytes are re-encoded through the wasm media jail
//!     ([`mw_render::media_jail::reencode_image`], t16-e5) to a metadata-stripped PNG
//!     before serving — a hostile codec never runs natively in this process;
//!   * results are cached by content hash (served with an `ETag`).
//!
//! The proxy REQUIRES a session ([`crate::authed`]) so it is never an open relay.
//!
//! # Ownership
//! This module exposes [`image_proxy_router`]; `crate::lib` (t16-e10, chain link 3)
//! MOUNTS it and applies [`SHELL_CSP_TIGHTENED`] at the shell-CSP site — this module
//! does not edit `lib.rs`.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::AppState;

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

/// Max bytes accepted from an upstream image (post-transfer-decoding). A hard
/// backstop against decompression/size bombs, enforced while streaming; the media
/// jail caps decode separately.
const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;
/// Per-request upstream timeout (applies to each redirect hop independently).
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
/// Maximum redirect hops followed (each re-validated); more → refuse.
const MAX_REDIRECTS: usize = 4;
/// Global concurrent-fetch ceiling — bounds proxy load + upstream fan-out.
const MAX_CONCURRENT: usize = 16;
/// Normalized outbound User-Agent; the reader's real UA is never forwarded.
const PROXY_UA: &str = "Mailwoman-Image-Proxy";
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

// ── SSRF refusal taxonomy ──────────────────────────────────────────────────────

/// Why a fetch was refused. All map to a client error or bad-gateway — an image
/// request never reveals internal reachability beyond a coarse status + reason.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Refusal {
    /// Malformed URL / non-http(s) scheme / credentials in URL / missing host.
    BadRequest(&'static str),
    /// The (only, or every resolved) target address is in a blocked range — the
    /// SSRF gate. A deliberately coarse `403` that does not distinguish "private"
    /// from "does not resolve".
    Blocked,
    /// Upstream too slow (per-hop timeout).
    Timeout,
    /// Upstream transport failure / non-success status / too many redirects.
    Upstream,
    /// Upstream body exceeded [`MAX_IMAGE_BYTES`].
    TooLarge,
}

impl IntoResponse for Refusal {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            Refusal::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            Refusal::Blocked => (StatusCode::FORBIDDEN, "target address is not permitted"),
            Refusal::Timeout => (StatusCode::GATEWAY_TIMEOUT, "upstream timed out"),
            Refusal::Upstream => (StatusCode::BAD_GATEWAY, "upstream fetch failed"),
            Refusal::TooLarge => (StatusCode::BAD_GATEWAY, "upstream image too large"),
        };
        (code, msg).into_response()
    }
}

// ── IP egress policy (DQ3) ─────────────────────────────────────────────────────

/// Whether an address is a permitted egress target: only globally-routable unicast.
/// Deny-by-default — anything loopback/private/link-local/ULA/multicast/reserved/
/// unspecified (incl. the cloud-metadata `169.254.169.254`) is refused. IPv4-mapped
/// IPv6 is unwrapped so `::ffff:127.0.0.1` cannot smuggle a loopback target.
fn ip_allowed(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => ipv4_allowed(v4),
        IpAddr::V6(v6) => {
            // Handle the v6-native specials FIRST: `::1`/`::` fall inside the
            // IPv4-compatible `::/96` block, so unwrapping them via `to_ipv4()`
            // before this check would route loopback/unspecified through the
            // permissive IPv4 path.
            if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() {
                return false;
            }
            // Unwrap IPv4-mapped (`::ffff:a.b.c.d`) and the deprecated IPv4-compatible
            // (`::a.b.c.d`) embeddings and re-check as IPv4, so e.g.
            // `::ffff:127.0.0.1` / `::7f00:1` cannot smuggle a loopback target.
            if let Some(v4) = v6.to_ipv4() {
                return ipv4_allowed(&v4);
            }
            ipv6_allowed(v6)
        }
    }
}

fn ipv4_allowed(ip: &Ipv4Addr) -> bool {
    if ip.is_loopback()          // 127.0.0.0/8
        || ip.is_private()       // 10/8, 172.16/12, 192.168/16
        || ip.is_link_local()    // 169.254/16 (incl. 169.254.169.254 metadata)
        || ip.is_unspecified()   // 0.0.0.0
        || ip.is_broadcast()     // 255.255.255.255
        || ip.is_multicast()     // 224/4
        || ip.is_documentation()
    // 192.0.2/24, 198.51.100/24, 203.0.113/24
    {
        return false;
    }
    let o = ip.octets();
    // CGNAT 100.64.0.0/10.
    if o[0] == 100 && (o[1] & 0xc0) == 0x40 {
        return false;
    }
    // Benchmarking 198.18.0.0/15.
    if o[0] == 198 && (o[1] & 0xfe) == 18 {
        return false;
    }
    // Reserved 240.0.0.0/4 (excludes the already-rejected 255.255.255.255).
    if o[0] >= 240 {
        return false;
    }
    // IETF protocol assignments 192.0.0.0/24 (incl. 192.0.0.0/29 etc).
    if o[0] == 192 && o[1] == 0 && o[2] == 0 {
        return false;
    }
    true
}

fn ipv6_allowed(ip: &Ipv6Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
        return false;
    }
    let seg = ip.segments();
    // Unique-local fc00::/7.
    if (seg[0] & 0xfe00) == 0xfc00 {
        return false;
    }
    // Link-local fe80::/10.
    if (seg[0] & 0xffc0) == 0xfe80 {
        return false;
    }
    // Documentation 2001:db8::/32.
    if seg[0] == 0x2001 && seg[1] == 0x0db8 {
        return false;
    }
    true
}

// ── validate + resolve (the SSRF gate) ─────────────────────────────────────────

/// A validated, IP-pinned fetch target.
#[derive(Debug)]
struct Target {
    /// The full URL to fetch (host unchanged so TLS SNI + Host match).
    url: reqwest::Url,
    /// The hostname (for the reqwest `resolve` pin).
    host: String,
    /// The single resolved, allowed socket address the fetch is pinned to.
    addr: SocketAddr,
}

/// Parse + validate a URL and resolve it to ONE allowed, pinned address. Refuses a
/// non-http(s) scheme, a URL with embedded credentials, a missing host, and any
/// target that resolves only to blocked ranges. DNS is resolved here exactly once;
/// the returned [`Target::addr`] is what the fetch connects to (anti-rebinding).
async fn validate_and_resolve(url: reqwest::Url) -> Result<Target, Refusal> {
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err(Refusal::BadRequest("only http/https URLs are proxied")),
    }
    // No credentials (DQ3).
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Refusal::BadRequest("credentials in URL are not allowed"));
    }
    let host = url
        .host_str()
        .filter(|h| !h.is_empty())
        .ok_or(Refusal::BadRequest("URL has no host"))?
        .to_string();
    let port = url
        .port_or_known_default()
        .ok_or(Refusal::BadRequest("URL has no port"))?;

    // Resolve ONCE. `lookup_host` parses an IP literal directly (so a literal
    // loopback/metadata host is caught here too). Pin to the first allowed address;
    // if none is allowed, refuse (a rebinding answer of [public, private] never
    // reaches the private one because we pin to the allowed address).
    let resolved = tokio::net::lookup_host((host.as_str(), port))
        .await
        .map_err(|_| Refusal::Blocked)?;
    let addr = resolved
        .into_iter()
        .find(|a| ip_allowed(&a.ip()))
        .ok_or(Refusal::Blocked)?;

    Ok(Target { url, host, addr })
}

// ── the pinned single-hop fetch ────────────────────────────────────────────────

/// One hop's result: either a validated body, or a redirect to re-validate.
#[derive(Debug)]
enum Hop {
    Body(Vec<u8>),
    Redirect(String),
}

/// Fetch ONE hop from a pinned target with redirects disabled + size/timeout caps.
/// The reqwest client pins `host → addr`, so even though the URL still names `host`
/// (for TLS/SNI/Host correctness) the connection goes only to the address we
/// validated. No cookie store; no forwarded headers.
async fn fetch_hop(target: &Target) -> Result<Hop, Refusal> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(FETCH_TIMEOUT)
        .resolve(&target.host, target.addr)
        .build()
        .map_err(|_| Refusal::Upstream)?;

    let resp = client
        .get(target.url.clone())
        .header(header::USER_AGENT, PROXY_UA)
        .header(header::ACCEPT, "image/*")
        // Ask for no transfer compression — one less decompression-bomb surface.
        .header(header::ACCEPT_ENCODING, "identity")
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                Refusal::Timeout
            } else {
                Refusal::Upstream
            }
        })?;

    let status = resp.status();
    if status.is_redirection() {
        let loc = resp
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(Refusal::Upstream)?
            .to_string();
        return Ok(Hop::Redirect(loc));
    }
    if !status.is_success() {
        return Err(Refusal::Upstream);
    }
    // Early size refusal from Content-Length when present.
    if let Some(len) = resp.content_length()
        && len as usize > MAX_IMAGE_BYTES
    {
        return Err(Refusal::TooLarge);
    }
    Ok(Hop::Body(read_capped(resp).await?))
}

/// Stream a response body, refusing once it exceeds [`MAX_IMAGE_BYTES`]. Uses
/// `chunk()` (no `stream` feature needed) so the cap applies to decoded bytes.
async fn read_capped(mut resp: reqwest::Response) -> Result<Vec<u8>, Refusal> {
    let mut buf = Vec::new();
    while let Some(chunk) = resp.chunk().await.map_err(|_| Refusal::Upstream)? {
        if buf.len() + chunk.len() > MAX_IMAGE_BYTES {
            return Err(Refusal::TooLarge);
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

/// Fetch a remote image, following (and re-validating) redirects. Every hop —
/// including each redirect target — goes through [`validate_and_resolve`], so a
/// redirect to a private/metadata address is refused exactly like a direct one.
async fn fetch_remote(start: reqwest::Url) -> Result<Vec<u8>, Refusal> {
    let mut url = start;
    for _ in 0..=MAX_REDIRECTS {
        let target = validate_and_resolve(url.clone()).await?;
        match fetch_hop(&target).await? {
            Hop::Body(bytes) => return Ok(bytes),
            Hop::Redirect(loc) => {
                // Resolve the Location against the current URL (handles relative
                // redirects) and loop — the new URL is re-validated next iteration.
                url = url.join(&loc).map_err(|_| Refusal::Upstream)?;
            }
        }
    }
    Err(Refusal::Upstream)
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
    // Require a session — never an open relay.
    if let Err(r) = crate::authed(&state, &headers).await {
        return r;
    }

    // Serve a cache hit before doing any work (and honor If-None-Match).
    if let Some((etag, png)) = cache().lock().expect("image cache lock").get(&q.url) {
        if if_none_match(&headers, &etag) {
            return not_modified(&etag);
        }
        return image_response(png, etag);
    }

    let url = match reqwest::Url::parse(&q.url) {
        Ok(u) => u,
        Err(_) => return Refusal::BadRequest("malformed URL").into_response(),
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
        Err(r) => return r.into_response(),
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

    // ── SSRF IP policy (the security core; pure, no network) ──────────────────

    #[test]
    fn blocks_loopback_private_linklocal_and_metadata() {
        for s in [
            "127.0.0.1",
            "127.5.6.7",
            "10.0.0.1",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.1.1",
            "169.254.169.254", // cloud metadata
            "0.0.0.0",
            "255.255.255.255",
            "224.0.0.1",  // multicast
            "100.64.0.1", // CGNAT
            "198.18.0.1", // benchmarking
            "240.0.0.1",  // reserved
            "192.0.2.1",  // documentation
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!ip_allowed(&ip), "{s} must be blocked");
        }
    }

    #[test]
    fn blocks_ipv6_loopback_ula_linklocal_and_mapped() {
        for s in [
            "::1",                    // loopback
            "::",                     // unspecified
            "fc00::1",                // ULA
            "fd12:3456::1",           // ULA
            "fe80::1",                // link-local
            "ff02::1",                // multicast
            "::ffff:127.0.0.1",       // IPv4-mapped loopback (must unwrap + block)
            "::ffff:169.254.169.254", // IPv4-mapped metadata
            "2001:db8::1",            // documentation
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!ip_allowed(&ip), "{s} must be blocked");
        }
    }

    #[test]
    fn allows_public_unicast() {
        for s in ["1.1.1.1", "8.8.8.8", "93.184.216.34", "2606:2800:220:1::1"] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(ip_allowed(&ip), "{s} must be allowed");
        }
    }

    // ── URL/scheme gate ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn refuses_non_http_schemes() {
        for u in ["file:///etc/passwd", "ftp://example.com/x", "gopher://x/1"] {
            let url = reqwest::Url::parse(u).unwrap();
            let err = validate_and_resolve(url).await.unwrap_err();
            assert!(matches!(err, Refusal::BadRequest(_)), "{u} → {err:?}");
        }
    }

    #[tokio::test]
    async fn refuses_credentials_in_url() {
        let url = reqwest::Url::parse("http://user:pw@example.com/x.png").unwrap();
        let err = validate_and_resolve(url).await.unwrap_err();
        assert!(matches!(err, Refusal::BadRequest(_)), "{err:?}");
    }

    #[tokio::test]
    async fn gate_blocks_literal_loopback_and_metadata_hosts() {
        // A literal private/metadata host is resolved by lookup_host to itself and
        // refused by the IP gate — no DNS needed. This is the end-to-end SSRF refusal.
        for u in [
            "http://127.0.0.1/x.png",
            "http://169.254.169.254/latest/meta-data/",
            "http://[::1]/x.png",
            "http://10.0.0.5/x.png",
        ] {
            let url = reqwest::Url::parse(u).unwrap();
            let err = validate_and_resolve(url).await.unwrap_err();
            assert_eq!(err, Refusal::Blocked, "{u} must be blocked");
        }
    }

    // ── fetch mechanics (size cap / timeout) against a LOCAL origin ────────────
    //
    // These call the low-level `fetch_hop` with a pinned loopback address DIRECTLY,
    // deliberately bypassing the SSRF gate (which — correctly — would refuse the
    // 127.0.0.1 test server). They exercise the streaming size cap + timeout, not
    // the gate (which the pure tests above cover).

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

    #[tokio::test]
    async fn fetch_hop_enforces_size_cap() {
        let big = vec![0u8; MAX_IMAGE_BYTES + 1];
        let addr = spawn_origin(big, None, StatusCode::OK, None).await;
        let err = fetch_hop(&target_for(addr)).await.unwrap_err();
        assert_eq!(err, Refusal::TooLarge);
    }

    #[tokio::test]
    async fn fetch_hop_returns_small_body() {
        let addr = spawn_origin(b"hello".to_vec(), None, StatusCode::OK, None).await;
        match fetch_hop(&target_for(addr)).await.unwrap() {
            Hop::Body(b) => assert_eq!(b, b"hello"),
            Hop::Redirect(_) => panic!("unexpected redirect"),
        }
    }

    #[tokio::test]
    async fn fetch_hop_surfaces_redirect_location() {
        let addr = spawn_origin(
            Vec::new(),
            None,
            StatusCode::FOUND,
            Some("http://127.0.0.1/next".into()),
        )
        .await;
        match fetch_hop(&target_for(addr)).await.unwrap() {
            Hop::Redirect(loc) => assert_eq!(loc, "http://127.0.0.1/next"),
            Hop::Body(_) => panic!("expected redirect"),
        }
    }

    #[tokio::test]
    async fn redirect_to_private_target_is_refused_by_the_loop() {
        // A public-looking start that 302s to a private host: the redirect is
        // re-validated and refused. We drive this at the loop level by joining +
        // re-validating the Location, matching `fetch_remote`'s hop check.
        let loc = "http://169.254.169.254/latest/";
        let joined = reqwest::Url::parse("http://cdn.example/x")
            .unwrap()
            .join(loc)
            .unwrap();
        let err = validate_and_resolve(joined).await.unwrap_err();
        assert_eq!(err, Refusal::Blocked);
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
        let bytes = match fetch_hop(&target_for(addr)).await.unwrap() {
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
