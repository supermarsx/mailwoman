//! V5 push relay + native bearer-auth (plan §2.2/§2.3/§3 e5).
//!
//! Everything here is ADDITIVE and OFF by default — the browser cookie/same-origin
//! path is byte-identical, and the native bearer / CORS modes only light up when a
//! client opts in (bearer token) or the operator sets `MW_NATIVE_ORIGINS`.
//!
//! Deliverables (plan §3 e5):
//!   * VAPID keygen on first boot ([`ensure_vapid`]) + `GET /api/push/vapid` serving
//!     the PUBLIC key (private sealed at rest via `mw_store::Store`),
//!   * `POST /api/push/subscribe|unsubscribe` over `mw_store::PushSubscriptionRow`,
//!   * the native bearer-auth helpers (token hashing + the `native_sessions` marker;
//!     bearer requests skip the cookie-only CSRF guard) + a config-gated CORS/origin
//!     allowlist,
//!   * the push DISPATCHER ([`run_dispatcher`]) = a second consumer of the engine
//!     `StateChange` broadcast that sends OPAQUE wakes (no message content) —
//!     WebPush (VAPID, RFC 8188) to browser/desktop endpoints, UnifiedPush (a plain
//!     POST) for Android, APNs mocked (needs an Apple account).

use anyhow::anyhow;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::broadcast;

use mw_engine::StateChange;
use mw_store::{PushSubscriptionRow, Store};

use crate::AppState;

/// The opaque wake payload. Push carries **NO message content** (plan §2.3 / risk
/// #8) — only this fixed marker whose sole meaning is "your account changed, wake
/// and re-fetch `/changes`". The subject/body of a message never transits push.
const WAKE_MARKER: &[u8] = b"mw-wake";

/// WebPush TTL (seconds) — how long the push service should hold an undelivered
/// wake. Four weeks: a wake that lands after the device comes back online still
/// triggers a refetch.
const WAKE_TTL: u32 = 2_419_200;

/// The Tauri shell origins the CORS/origin allowlist opts into when configured
/// (plan §2.2). Threaded from env `MW_NATIVE_ORIGINS` (comma-separated, default
/// EMPTY → off): with no origins configured, no CORS headers are emitted and
/// browser deployments see no behavior change.
#[derive(Debug, Clone, Default)]
pub struct NativeAuthConfig {
    /// Allowed shell origins (e.g. `tauri://localhost`, `https://tauri.localhost`).
    /// Empty = the native/CORS mode is OFF (the default).
    pub origins: Vec<String>,
}

impl NativeAuthConfig {
    /// Populate from `MW_NATIVE_ORIGINS` (comma-separated). Absent/empty → off.
    pub fn from_env() -> Self {
        let origins = std::env::var("MW_NATIVE_ORIGINS")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        Self { origins }
    }

    /// Whether the native/CORS mode is enabled (any origin configured).
    pub fn is_enabled(&self) -> bool {
        !self.origins.is_empty()
    }

    /// Whether `origin` is on the allowlist (exact match).
    pub fn allows(&self, origin: &str) -> bool {
        self.origins.iter().any(|o| o == origin)
    }
}

/// Extract a `Authorization: Bearer <token>` value, if present. The seam the
/// bearer-accept path uses (in addition to the cookie) on the authed routes;
/// bearer requests skip the cookie-only CSRF guard (no ambient authority to
/// protect — origin-agnostic).
pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))?;
    let token = token.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// Hash a bearer token for the `native_sessions` marker table. Only the HASH is
/// stored (plan §2.4), never the token itself.
pub(crate) fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let digest = hasher.finalize();
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ---------------------------------------------------------------------------
// VAPID keypair (generated on first boot; private sealed at rest)
// ---------------------------------------------------------------------------

/// Ensure a VAPID keypair exists, generating + persisting one (private SEALED via
/// the store's `ServerKey`) on first boot. Idempotent: a no-op once a keypair is
/// stored. Called from `build_app` so `GET /api/push/vapid` and the dispatcher
/// always have a key.
pub(crate) async fn ensure_vapid(store: &Store) -> anyhow::Result<()> {
    if store.vapid_public().await?.is_some() {
        return Ok(());
    }
    let (public_b64, private_scalar) = generate_vapid_keypair()?;
    let created_at = now_rfc3339();
    store
        .store_vapid_keypair(&public_b64, &private_scalar, &created_at)
        .await?;
    tracing::info!("generated a fresh VAPID keypair for Web Push");
    Ok(())
}

/// Generate a P-256 VAPID keypair. Returns `(applicationServerKey, private scalar)`:
/// the public key is the base64url (no-pad) of the uncompressed SEC1 point (what the
/// browser passes to `pushManager.subscribe`); the private key is the raw 32-byte
/// EC scalar (what `jwt_simple::ES256KeyPair::from_bytes` consumes), only ever
/// stored SEALED. Uses `web-push-native`'s re-exported `p256` so the curve version
/// matches the signer exactly.
fn generate_vapid_keypair() -> anyhow::Result<(String, Vec<u8>)> {
    use web_push_native::p256;
    use web_push_native::p256::elliptic_curve::sec1::ToEncodedPoint;

    // A valid non-zero scalar < group order; retry on the (astronomically rare)
    // invalid draw rather than coupling to a specific RNG-crate version.
    let secret = loop {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
        if let Ok(sk) = p256::SecretKey::from_slice(&bytes) {
            break sk;
        }
    };
    let point = secret.public_key().to_encoded_point(false);
    let public_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(point.as_bytes());
    Ok((public_b64, secret.to_bytes().to_vec()))
}

// ---------------------------------------------------------------------------
// HTTP endpoints
// ---------------------------------------------------------------------------

/// `GET /api/push/vapid` → `{ publicKey }` (public-only). Cookie- or bearer-authed
/// like every other endpoint; the browser needs this key to subscribe in-page.
pub(crate) async fn push_vapid(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    match state.store.vapid_public().await {
        Ok(Some(public_key)) => Json(json!({ "publicKey": public_key })).into_response(),
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "VAPID key not initialized" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("VAPID public key load failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
        }
    }
}

/// The Web-Push subscription keys (§2.3), present for the `webpush` transport only.
#[derive(Debug, Deserialize)]
struct PushKeys {
    p256dh: String,
    auth: String,
}

/// `PushSubscriptionInfo` request body (plan §2.3). camelCase over the wire.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscribeReq {
    transport: String,
    endpoint: String,
    #[serde(default)]
    keys: Option<PushKeys>,
    #[serde(default)]
    app_id: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
}

/// `POST /api/push/subscribe` (body = `PushSubscriptionInfo`) → `{ id, vapidPublicKey }`.
/// Stores the subscription idempotently (keyed by endpoint). Cookie- or bearer-authed.
pub(crate) async fn push_subscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let req: SubscribeReq = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("invalid subscription: {e}") })),
            )
                .into_response();
        }
    };
    if !matches!(req.transport.as_str(), "webpush" | "unifiedpush" | "apns") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "transport must be webpush|unifiedpush|apns" })),
        )
            .into_response();
    }
    if req.transport == "webpush" && req.keys.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "webpush subscriptions require keys.p256dh + keys.auth" })),
        )
            .into_response();
    }
    let (p256dh, auth) = match req.keys {
        Some(k) => (Some(k.p256dh), Some(k.auth)),
        None => (None, None),
    };
    let row = PushSubscriptionRow {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: session.account_id.clone(),
        transport: req.transport,
        endpoint: req.endpoint,
        p256dh,
        auth,
        app_id: req.app_id,
        expires_at: req.expires_at,
        created_at: now_rfc3339(),
        last_wake_at: None,
    };
    if let Err(e) = state.store.upsert_push_subscription(&row).await {
        tracing::error!("failed to store push subscription: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response();
    }
    let vapid_public = state.store.vapid_public().await.ok().flatten();
    Json(json!({ "id": row.id, "vapidPublicKey": vapid_public })).into_response()
}

/// `POST /api/push/unsubscribe {id|endpoint}`. Removes the subscription. Cookie- or
/// bearer-authed.
#[derive(Debug, Deserialize)]
struct UnsubscribeReq {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    endpoint: Option<String>,
}

pub(crate) async fn push_unsubscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let req: UnsubscribeReq = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("invalid request: {e}") })),
            )
                .into_response();
        }
    };
    let outcome = match (&req.endpoint, &req.id) {
        (Some(endpoint), _) => state.store.delete_push_subscription(endpoint).await,
        (None, Some(id)) => state.store.delete_push_subscription_by_id(id).await,
        (None, None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "id or endpoint required" })),
            )
                .into_response();
        }
    };
    match outcome {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("failed to remove push subscription: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Push dispatcher — second consumer of the engine StateChange broadcast
// ---------------------------------------------------------------------------

/// Drain the push-relay broadcast for the life of the process, sending an opaque
/// wake to every active subscription of a changed account (plan §2.3). Spawned
/// once by `build_app`. Never carries message content — the wake only prompts the
/// client to foreground-fetch `/changes` (the same refetch the WS/SSE path does).
pub(crate) async fn run_dispatcher(
    store: Store,
    mut rx: broadcast::Receiver<StateChange>,
    http: reqwest::Client,
) {
    loop {
        match rx.recv().await {
            Ok(change) => dispatch_wakes(&store, &http, &change).await,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("push dispatcher lagged {n} changes");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

/// Fan an account change out to its subscriptions as opaque wakes, respecting a
/// (cheap, optional) global quiet-hours window (§17.3).
async fn dispatch_wakes(store: &Store, http: &reqwest::Client, change: &StateChange) {
    if quiet_now(store).await {
        return;
    }
    let subs = match store.list_push_subscriptions(&change.account_id).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("push dispatcher: subscription lookup failed: {e}");
            return;
        }
    };
    if subs.is_empty() {
        return;
    }
    // Load the sealed VAPID keypair once per change (only the WebPush transport
    // needs it). `None` before first-boot init — WebPush is skipped, others proceed.
    let vapid = store.load_vapid_keypair().await.ok().flatten();
    for sub in subs {
        let result = match sub.transport.as_str() {
            "webpush" => send_webpush(http, &sub, vapid.as_ref()).await,
            "unifiedpush" => send_unifiedpush(http, &sub).await,
            "apns" => send_apns_mock(&sub),
            other => {
                tracing::warn!("push dispatcher: unknown transport {other:?}");
                continue;
            }
        };
        match result {
            // Gone / not found: the endpoint expired — drop the subscription.
            Ok(status) if status == 404 || status == 410 => {
                let _ = store.delete_push_subscription(&sub.endpoint).await;
            }
            Ok(_) => {
                let _ = store.touch_push_wake(&sub.id, &now_rfc3339()).await;
            }
            Err(e) => tracing::warn!("push send to {} failed: {e}", sub.endpoint),
        }
    }
}

/// Send a VAPID-signed, RFC 8188-encrypted opaque wake to a Web Push endpoint.
/// `web-push-native` builds the encrypted, VAPID-authenticated `http::Request`
/// (all headers + ECE body); we transmit it over the in-tree `reqwest` client (no
/// extra HTTP stack). The encrypted body is the fixed [`WAKE_MARKER`] — never
/// message content.
async fn send_webpush(
    http: &reqwest::Client,
    sub: &PushSubscriptionRow,
    vapid: Option<&(String, Vec<u8>)>,
) -> anyhow::Result<u16> {
    use web_push_native::jwt_simple::algorithms::ES256KeyPair;
    use web_push_native::p256::PublicKey;
    use web_push_native::{Auth, WebPushBuilder};

    let Some((_public, private_scalar)) = vapid else {
        anyhow::bail!("no VAPID keypair available");
    };
    let (Some(p256dh), Some(auth)) = (sub.p256dh.as_deref(), sub.auth.as_deref()) else {
        anyhow::bail!("webpush subscription missing p256dh/auth");
    };

    let ua_public =
        PublicKey::from_sec1_bytes(&b64url_decode(p256dh)?).map_err(|e| anyhow!("p256dh: {e}"))?;
    let auth_bytes = b64url_decode(auth)?;
    if auth_bytes.len() != 16 {
        anyhow::bail!("webpush auth secret must be 16 bytes");
    }
    let ua_auth = Auth::clone_from_slice(&auth_bytes);
    let key_pair =
        ES256KeyPair::from_bytes(private_scalar).map_err(|e| anyhow!("VAPID key load: {e}"))?;
    let endpoint: axum::http::Uri = sub.endpoint.parse().map_err(|e| anyhow!("endpoint: {e}"))?;

    let request = WebPushBuilder::new(endpoint, ua_public, ua_auth)
        .with_valid_duration(std::time::Duration::from_secs(u64::from(WAKE_TTL)))
        .with_vapid(&key_pair, &vapid_contact())
        .build(WAKE_MARKER.to_vec())
        .map_err(|e| anyhow!("web push build: {e}"))?;

    // Translate the built `http::Request` (POST + ECE headers + VAPID auth + body)
    // into a `reqwest` request — the http crate is shared with axum/reqwest.
    let (parts, body) = request.into_parts();
    let mut req = http.post(parts.uri.to_string()).body(body);
    for (name, value) in parts.headers.iter() {
        req = req.header(name.clone(), value.clone());
    }
    let resp = req.send().await?;
    Ok(resp.status().as_u16())
}

/// Decode a base64url subscription field (browsers may or may not pad).
fn b64url_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD
        .decode(s)
        .or_else(|_| URL_SAFE.decode(s))
        .map_err(|e| anyhow!("base64url: {e}"))
}

/// Send an opaque wake to a UnifiedPush endpoint (Android, self-hostable, no
/// Google). UnifiedPush delivers the POST body to the app; the app treats any
/// delivery as "sync now", so the body is the fixed [`WAKE_MARKER`] — no content.
async fn send_unifiedpush(
    http: &reqwest::Client,
    sub: &PushSubscriptionRow,
) -> anyhow::Result<u16> {
    let resp = http.post(&sub.endpoint).body(WAKE_MARKER).send().await?;
    Ok(resp.status().as_u16())
}

/// APNs opaque wake — MOCKED (plan §1.7 / risk #5): live APNs needs an Apple
/// provider certificate + account, unavailable on this Windows CI. A real send
/// would be a `content-available` background push carrying NO content. Recorded
/// here so the dispatcher path is exercised end-to-end without Apple infra.
fn send_apns_mock(sub: &PushSubscriptionRow) -> anyhow::Result<u16> {
    tracing::info!(
        "APNs opaque wake (mocked) for endpoint {} — no content transits push",
        sub.endpoint
    );
    Ok(200)
}

/// The VAPID `sub` claim (a contact URI push services use to reach the operator on
/// delivery problems). From `MW_VAPID_CONTACT`, else a neutral default.
fn vapid_contact() -> String {
    std::env::var("MW_VAPID_CONTACT")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "mailto:push@mailwoman.invalid".to_string())
}

// ---------------------------------------------------------------------------
// Quiet hours (cheap, optional — §17.3)
// ---------------------------------------------------------------------------

/// Whether the current UTC hour falls in the operator's quiet-hours window. The
/// window is a `"start-end"` (UTC hours) `settings` value under `push.quiet_hours`;
/// absent/unparseable → never quiet (the default). One indexed settings read per
/// change — cheap enough to honor per the plan's "if cheap".
async fn quiet_now(store: &Store) -> bool {
    let Ok(Some(raw)) = store.get_setting("push.quiet_hours").await else {
        return false;
    };
    let Some((start, end)) = parse_quiet_window(&raw) else {
        return false;
    };
    in_quiet_window(start, end, current_utc_hour())
}

/// Parse a `"start-end"` (24h UTC) quiet-hours window, e.g. `"22-7"`.
fn parse_quiet_window(raw: &str) -> Option<(u32, u32)> {
    let (start, end) = raw.trim().split_once('-')?;
    let start: u32 = start.trim().parse().ok()?;
    let end: u32 = end.trim().parse().ok()?;
    (start < 24 && end < 24).then_some((start, end))
}

/// Whether `hour` is inside `[start, end)`, wrapping past midnight when `start > end`.
fn in_quiet_window(start: u32, end: u32, hour: u32) -> bool {
    if start <= end {
        hour >= start && hour < end
    } else {
        hour >= start || hour < end
    }
}

fn current_utc_hour() -> u32 {
    use chrono::Timelike;
    chrono::Utc::now().hour()
}

/// An RFC 3339 timestamp for the store's textual `*_at` columns.
pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn bearer_token_parses_case_insensitively() {
        let mut h = HeaderMap::new();
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer abc123"),
        );
        assert_eq!(bearer_token(&h).as_deref(), Some("abc123"));
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("bearer  xyz "),
        );
        assert_eq!(bearer_token(&h).as_deref(), Some("xyz"));
    }

    #[test]
    fn bearer_token_absent_without_header() {
        assert!(bearer_token(&HeaderMap::new()).is_none());
    }

    #[test]
    fn native_auth_off_by_default() {
        assert!(!NativeAuthConfig::default().is_enabled());
    }

    #[test]
    fn native_auth_parses_origins() {
        let cfg = NativeAuthConfig {
            origins: vec!["tauri://localhost".into()],
        };
        assert!(cfg.is_enabled());
        assert!(cfg.allows("tauri://localhost"));
        assert!(!cfg.allows("https://evil.example"));
    }

    #[test]
    fn token_hash_is_stable_and_not_the_token() {
        let h1 = hash_token("secret-token");
        let h2 = hash_token("secret-token");
        assert_eq!(h1, h2);
        assert_ne!(h1, "secret-token");
        assert_eq!(h1.len(), 64); // SHA-256 hex
    }

    #[test]
    fn generated_vapid_public_is_uncompressed_p256() {
        let (public_b64, scalar) = generate_vapid_keypair().unwrap();
        let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&public_b64)
            .unwrap();
        // Uncompressed SEC1 point: 0x04 || X(32) || Y(32).
        assert_eq!(raw.len(), 65);
        assert_eq!(raw[0], 0x04);
        // The stored private scalar is the raw 32-byte EC key the VAPID signer loads.
        assert_eq!(scalar.len(), 32);
        assert!(web_push_native::jwt_simple::algorithms::ES256KeyPair::from_bytes(&scalar).is_ok());
    }

    #[test]
    fn vapid_signed_webpush_request_is_built_over_reqwest() {
        // The full WebPush build path (VAPID-signed, ECE-encrypted) produces a POST
        // to the endpoint whose plaintext body never appears (it is encrypted, and
        // in any case carries only the opaque marker — no message content).
        use web_push_native::jwt_simple::algorithms::ES256KeyPair;
        use web_push_native::p256::SecretKey;
        use web_push_native::{Auth, WebPushBuilder};

        let (_public, scalar) = generate_vapid_keypair().unwrap();
        let key_pair = ES256KeyPair::from_bytes(&scalar).unwrap();
        // A well-formed peer (browser) subscription key + 16-byte auth secret.
        let ua_secret = loop {
            let mut b = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut b);
            if let Ok(sk) = SecretKey::from_slice(&b) {
                break sk;
            }
        };
        let ua_public = ua_secret.public_key();
        let ua_auth = Auth::clone_from_slice(&[7u8; 16]);
        let request = WebPushBuilder::new(
            "https://push.example/abc".parse().unwrap(),
            ua_public,
            ua_auth,
        )
        .with_vapid(&key_pair, "mailto:push@mailwoman.invalid")
        .build(WAKE_MARKER.to_vec())
        .unwrap();
        assert_eq!(request.method(), axum::http::Method::POST);
        assert!(
            request
                .headers()
                .contains_key(axum::http::header::AUTHORIZATION)
        );
        // The encrypted body is not the plaintext marker.
        assert_ne!(request.body().as_slice(), WAKE_MARKER);
    }

    #[test]
    fn quiet_window_parsing_and_membership() {
        assert_eq!(parse_quiet_window("22-7"), Some((22, 7)));
        assert_eq!(parse_quiet_window(" 9 - 17 "), Some((9, 17)));
        assert_eq!(parse_quiet_window("nonsense"), None);
        assert_eq!(parse_quiet_window("25-7"), None);
        // Wrapping window 22:00–07:00.
        assert!(in_quiet_window(22, 7, 23));
        assert!(in_quiet_window(22, 7, 3));
        assert!(!in_quiet_window(22, 7, 12));
        // Same-day window 09:00–17:00.
        assert!(in_quiet_window(9, 17, 12));
        assert!(!in_quiet_window(9, 17, 8));
        assert!(!in_quiet_window(9, 17, 17));
    }
}
