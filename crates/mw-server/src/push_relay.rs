//! V5 push relay + native bearer-auth (plan §2.2/§2.3/§3 e5).
//!
//! Everything here is ADDITIVE — the browser cookie/same-origin path is
//! byte-identical. The native bearer mode lights up when a client opts in (bearer
//! token); CORS headers go only to the native shell origins ([`TAURI_ORIGINS`] plus
//! `MW_NATIVE_ORIGINS`), never to a browser origin.
//!
//! Deliverables (plan §3 e5):
//!   * VAPID keygen on first boot ([`ensure_vapid`]) + `GET /api/push/vapid` serving
//!     the PUBLIC key (private sealed at rest via `mw_store::Store`),
//!   * `POST /api/push/subscribe|unsubscribe` over `mw_store::PushSubscriptionRow`,
//!   * the native bearer-auth helpers (token hashing + the `native_sessions` marker;
//!     bearer requests skip the cookie-only CSRF guard) + the native-origin
//!     allowlist for CORS and cookie-less sign-in,
//!   * the push DISPATCHER ([`run_dispatcher`]) = a second consumer of the engine
//!     `StateChange` broadcast that sends OPAQUE wakes (no message content) —
//!     WebPush (RFC 8291 encryption, RFC 8292 VAPID) to browser/desktop endpoints,
//!     UnifiedPush (a plain POST) for Android, APNs mocked (needs an Apple account).

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

/// The RFC 8188 record size written into the `aes128gcm` header. One record holds
/// the whole payload, so a payload may be at most `RECORD_SIZE - 17` bytes (one
/// delimiter byte and the 16-byte tag).
const RECORD_SIZE: u32 = 4096;

/// Lifetime of the VAPID JWT. RFC 8292 §2 caps `exp` at 24 hours after the
/// request (push services refuse longer); 12 hours leaves room for clock skew.
/// It is unrelated to [`WAKE_TTL`], which is how long the push service holds a wake.
const VAPID_JWT_LIFETIME_SECS: i64 = 12 * 60 * 60;

/// The origins a Tauri 2 webview loads the SPA from: `http://tauri.localhost` on
/// Windows and Android, `tauri://localhost` on macOS, iOS and Linux, and
/// `https://tauri.localhost` when a shell sets `useHttpsScheme`. The native shells
/// send one of these as `Origin` on every request to the user's server.
///
/// They are accepted by default (t31, decision D10) and nothing else is. A web page
/// cannot send any of them: a browser sets `Origin` from the page's own scheme and
/// host, and no page is served from `tauri:` or from the `tauri.localhost` host a
/// webview resolves internally. Accepting them therefore opens nothing to
/// cross-site requests from the web.
pub const TAURI_ORIGINS: [&str; 3] = [
    "http://tauri.localhost",
    "https://tauri.localhost",
    "tauri://localhost",
];

/// The native shell origins the server answers with CORS headers and lets sign in
/// without a cookie: [`TAURI_ORIGINS`], plus any the operator lists in
/// `MW_NATIVE_ORIGINS` (comma-separated, e.g. for a shell with another scheme).
/// A browser deployment is not affected: no browser origin is on the list, so a
/// browser request never gets an `Access-Control-*` header.
#[derive(Debug, Clone, Default)]
pub struct NativeAuthConfig {
    /// Origins `MW_NATIVE_ORIGINS` adds to [`TAURI_ORIGINS`]. Empty by default.
    pub extra_origins: Vec<String>,
}

impl NativeAuthConfig {
    /// Populate from `MW_NATIVE_ORIGINS` (comma-separated). Absent/empty → only
    /// [`TAURI_ORIGINS`].
    pub fn from_env() -> Self {
        let extra_origins = std::env::var("MW_NATIVE_ORIGINS")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        Self { extra_origins }
    }

    /// Whether `origin` is a native shell origin (exact match).
    pub fn allows(&self, origin: &str) -> bool {
        TAURI_ORIGINS.contains(&origin) || self.extra_origins.iter().any(|o| o == origin)
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
/// EC scalar (what [`vapid_authorization`] signs with), only ever stored SEALED.
fn generate_vapid_keypair() -> anyhow::Result<(String, Vec<u8>)> {
    use p256::elliptic_curve::sec1::ToSec1Point;

    let secret = random_p256_secret();
    let point = secret.public_key().to_sec1_point(false);
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

/// Send a VAPID-signed, RFC 8291-encrypted opaque wake to a Web Push endpoint.
/// [`build_wake_request`] builds the `http::Request` (headers + encrypted body);
/// it goes out over the in-tree `reqwest` client. The encrypted body is the fixed
/// [`WAKE_MARKER`] — never message content.
async fn send_webpush(
    http: &reqwest::Client,
    sub: &PushSubscriptionRow,
    vapid: Option<&(String, Vec<u8>)>,
) -> anyhow::Result<u16> {
    let Some((_public, private_scalar)) = vapid else {
        anyhow::bail!("no VAPID keypair available");
    };
    let (Some(p256dh), Some(auth)) = (sub.p256dh.as_deref(), sub.auth.as_deref()) else {
        anyhow::bail!("webpush subscription missing p256dh/auth");
    };
    let request = build_wake_request(
        &sub.endpoint,
        p256dh,
        auth,
        private_scalar,
        &vapid_contact(),
        WAKE_MARKER,
    )?;

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

/// Build the encrypted, VAPID-authenticated Web Push request for one subscription:
/// the body is `payload` encrypted to the subscriber's key (RFC 8291 over RFC 8188
/// `aes128gcm`), the `Authorization` header is a VAPID ES256 JWT (RFC 8292).
fn build_wake_request(
    endpoint: &str,
    p256dh: &str,
    auth: &str,
    vapid_private: &[u8],
    contact: &str,
    payload: &[u8],
) -> anyhow::Result<axum::http::Request<Vec<u8>>> {
    let uri: axum::http::Uri = endpoint.parse().map_err(|e| anyhow!("endpoint: {e}"))?;
    let audience = push_origin(&uri)?;
    let ua_public = p256::PublicKey::from_sec1_bytes(&b64url_decode(p256dh)?)
        .map_err(|e| anyhow!("p256dh: {e}"))?;
    let auth = b64url_decode(auth)?;
    if auth.len() != 16 {
        anyhow::bail!("webpush auth secret must be 16 bytes");
    }
    let vapid_key =
        p256::SecretKey::from_slice(vapid_private).map_err(|e| anyhow!("VAPID key load: {e}"))?;

    let mut salt = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
    let body = encrypt_aes128gcm(payload, &random_p256_secret(), &ua_public, &auth, &salt)?;
    let expires = chrono::Utc::now().timestamp() + VAPID_JWT_LIFETIME_SECS;
    let authorization = vapid_authorization(&vapid_key, &audience, contact, expires)?;

    axum::http::Request::builder()
        .method(axum::http::Method::POST)
        .uri(uri)
        .header("TTL", WAKE_TTL)
        .header(header::CONTENT_ENCODING, "aes128gcm")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::AUTHORIZATION, authorization)
        .body(body)
        .map_err(|e| anyhow!("web push request: {e}"))
}

/// The RFC 6454 origin of a push endpoint, the VAPID `aud` claim (RFC 8292 §2):
/// scheme and host, plus the port when it is not the scheme's default.
fn push_origin(uri: &axum::http::Uri) -> anyhow::Result<String> {
    let scheme = uri
        .scheme_str()
        .ok_or_else(|| anyhow!("endpoint: no scheme"))?;
    let default_port = match scheme {
        "https" => 443,
        "http" => 80,
        other => anyhow::bail!("endpoint: unsupported scheme {other:?}"),
    };
    let host = uri.host().ok_or_else(|| anyhow!("endpoint: no host"))?;
    Ok(match uri.port_u16() {
        Some(port) if port != default_port => format!("{scheme}://{host}:{port}"),
        _ => format!("{scheme}://{host}"),
    })
}

/// RFC 8291 §3.4 encryption into one RFC 8188 `aes128gcm` record:
/// `salt(16) ‖ rs(4) ‖ idlen(1) ‖ keyid(65 = the sender's ephemeral public key) ‖
/// AES-128-GCM(payload ‖ 0x02)`. `as_secret` is a fresh key per message.
fn encrypt_aes128gcm(
    payload: &[u8],
    as_secret: &p256::SecretKey,
    ua_public: &p256::PublicKey,
    auth: &[u8],
    salt: &[u8; 16],
) -> anyhow::Result<Vec<u8>> {
    use aes_gcm::aead::{Aead, KeyInit};
    use hkdf::Hkdf;
    use p256::elliptic_curve::sec1::ToSec1Point;

    // One record holds the payload, the delimiter byte and the 16-byte tag.
    if payload.len() + 17 > RECORD_SIZE as usize {
        anyhow::bail!("web push payload too large");
    }
    let ua_point = ua_public.to_sec1_point(false);
    let as_point = as_secret.public_key().to_sec1_point(false);
    let shared = p256::ecdh::diffie_hellman(as_secret.to_nonzero_scalar(), ua_public.as_affine());

    // IKM = HKDF(auth, ecdh_secret, "WebPush: info" ‖ 0 ‖ ua_public ‖ as_public, 32).
    let mut key_info = Vec::with_capacity(14 + 65 + 65);
    key_info.extend_from_slice(b"WebPush: info\0");
    key_info.extend_from_slice(ua_point.as_bytes());
    key_info.extend_from_slice(as_point.as_bytes());
    let mut ikm = [0u8; 32];
    Hkdf::<Sha256>::new(Some(auth), shared.raw_secret_bytes().as_slice())
        .expand(&key_info, &mut ikm)
        .map_err(|e| anyhow!("hkdf: {e}"))?;

    // CEK and nonce from the record salt (RFC 8188 §2.2, §2.3).
    let prk = Hkdf::<Sha256>::new(Some(salt), &ikm);
    let mut cek = [0u8; 16];
    prk.expand(b"Content-Encoding: aes128gcm\0", &mut cek)
        .map_err(|e| anyhow!("hkdf: {e}"))?;
    let mut nonce = [0u8; 12];
    prk.expand(b"Content-Encoding: nonce\0", &mut nonce)
        .map_err(|e| anyhow!("hkdf: {e}"))?;

    let mut record = Vec::with_capacity(payload.len() + 1);
    record.extend_from_slice(payload);
    record.push(0x02); // last-record delimiter, no padding
    let sealed = aes_gcm::Aes128Gcm::new_from_slice(&cek)
        .map_err(|e| anyhow!("aes-gcm key: {e}"))?
        .encrypt(&nonce.into(), record.as_slice())
        .map_err(|e| anyhow!("aes-gcm: {e}"))?;

    let mut out = Vec::with_capacity(21 + 65 + sealed.len());
    out.extend_from_slice(salt);
    out.extend_from_slice(&RECORD_SIZE.to_be_bytes());
    out.push(65);
    out.extend_from_slice(as_point.as_bytes());
    out.extend_from_slice(&sealed);
    Ok(out)
}

/// The RFC 8292 §3 header value `vapid t=<JWT>, k=<public key>`: an ES256 JWT
/// (raw r ‖ s signature) with `aud`, `exp` and `sub`, and the base64url
/// uncompressed public key it verifies with.
fn vapid_authorization(
    key: &p256::SecretKey,
    audience: &str,
    contact: &str,
    expires: i64,
) -> anyhow::Result<String> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use p256::ecdsa::signature::Signer;
    use p256::elliptic_curve::sec1::ToSec1Point;

    let head = URL_SAFE_NO_PAD.encode(br#"{"typ":"JWT","alg":"ES256"}"#);
    let claims = serde_json::to_vec(&json!({ "aud": audience, "exp": expires, "sub": contact }))?;
    let signing_input = format!("{head}.{}", URL_SAFE_NO_PAD.encode(claims));
    let signature: p256::ecdsa::Signature =
        p256::ecdsa::SigningKey::from(key).sign(signing_input.as_bytes());
    let public = URL_SAFE_NO_PAD.encode(key.public_key().to_sec1_point(false).as_bytes());
    Ok(format!(
        "vapid t={signing_input}.{}, k={public}",
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    ))
}

/// A uniformly random P-256 secret key from the thread CSPRNG.
fn random_p256_secret() -> p256::SecretKey {
    // A valid non-zero scalar < group order; retry on the (astronomically rare)
    // invalid draw rather than coupling to a specific RNG-crate version.
    loop {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
        if let Ok(sk) = p256::SecretKey::from_slice(&bytes) {
            return sk;
        }
    }
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
    fn native_auth_allows_only_the_tauri_origins_by_default() {
        let cfg = NativeAuthConfig::default();
        for origin in TAURI_ORIGINS {
            assert!(cfg.allows(origin), "{origin}");
        }
        for origin in [
            "https://evil.example",
            "http://tauri.localhost.evil.example",
            "http://tauri.localhost:8080",
            "tauri://evil",
            "TAURI://LOCALHOST",
            "",
        ] {
            assert!(!cfg.allows(origin), "{origin}");
        }
    }

    #[test]
    fn native_auth_adds_the_configured_origins() {
        let cfg = NativeAuthConfig {
            extra_origins: vec!["app://mailwoman".into()],
        };
        assert!(cfg.allows("app://mailwoman"));
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
        assert!(p256::ecdsa::SigningKey::from_slice(&scalar).is_ok());
    }

    #[test]
    fn encryption_matches_the_rfc_8291_example() {
        // RFC 8291 Appendix A: fixed sender key, salt and record size 4096 give this
        // exact message body.
        let d = |v: &str| {
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(v)
                .unwrap()
        };
        let as_secret =
            p256::SecretKey::from_slice(&d("yfWPiYE-n46HLnH0KqZOF1fJJU3MYrct3AELtAQ-oRw")).unwrap();
        let ua_public = p256::PublicKey::from_sec1_bytes(&d(
            "BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4",
        ))
        .unwrap();
        let ua_secret =
            p256::SecretKey::from_slice(&d("q1dXpw3UpT5VOmu_cf_v6ih07Aems3njxI-JWgLcM94")).unwrap();
        let auth = d("BTBZMqHH6r4Tts7J_aSIgg");
        let salt: [u8; 16] = d("DGv6ra1nlYgDCS1FRnbzlw").try_into().unwrap();
        let plaintext = b"When I grow up, I want to be a watermelon";
        let body = encrypt_aes128gcm(plaintext, &as_secret, &ua_public, &auth, &salt).unwrap();
        assert_eq!(
            body,
            d(
                "DGv6ra1nlYgDCS1FRnbzlwAAEABBBP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A_yl95bQpu6cVPTpK4Mqgkf1CXztLVBSt2Ks3oZwbuwXPXLWyouBWLVWGNWQexSgSxsj_Qulcy4a-fN"
            )
        );
        assert_eq!(decrypt_wake(&body, &ua_secret, &auth), plaintext);
    }

    #[test]
    fn encryption_refuses_a_payload_over_one_record() {
        let ua_public = random_p256_secret().public_key();
        let fits = vec![0u8; RECORD_SIZE as usize - 17];
        let too_big = vec![0u8; RECORD_SIZE as usize - 16];
        let k = random_p256_secret();
        assert!(encrypt_aes128gcm(&fits, &k, &ua_public, &[0u8; 16], &[0u8; 16]).is_ok());
        assert!(encrypt_aes128gcm(&too_big, &k, &ua_public, &[0u8; 16], &[0u8; 16]).is_err());
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

    // ── Receiver-side checks (written from RFC 8291 / RFC 8292, not from the
    // sender code): decrypt the body with the subscriber's key and verify the VAPID
    // JWT with the server's public key.

    use aes_gcm::aead::{Aead, KeyInit};
    use hkdf::Hkdf;
    use p256::elliptic_curve::sec1::ToSec1Point;

    const CONTACT: &str = "mailto:ops@example.test";

    fn ua_keypair() -> (p256::SecretKey, String) {
        let secret = loop {
            let mut b = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut b);
            if let Ok(sk) = p256::SecretKey::from_slice(&b) {
                break sk;
            }
        };
        let public = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(secret.public_key().to_sec1_point(false).as_bytes());
        (secret, public)
    }

    fn b64(s: &str) -> Vec<u8> {
        b64url_decode(s).unwrap()
    }

    /// RFC 8291 §3.4 + RFC 8188 §2: one aes128gcm record, last-record delimiter 0x02.
    fn decrypt_wake(body: &[u8], ua_secret: &p256::SecretKey, auth: &[u8]) -> Vec<u8> {
        let salt = &body[..16];
        let rs = u32::from_be_bytes(body[16..20].try_into().unwrap()) as usize;
        let idlen = usize::from(body[20]);
        let keyid = &body[21..21 + idlen];
        let record = &body[21 + idlen..];
        assert!(record.len() <= rs, "one record of at most rs bytes");
        let as_public = p256::PublicKey::from_sec1_bytes(keyid).unwrap();
        let shared =
            p256::ecdh::diffie_hellman(ua_secret.to_nonzero_scalar(), as_public.as_affine());
        let mut key_info = b"WebPush: info\0".to_vec();
        key_info.extend_from_slice(ua_secret.public_key().to_sec1_point(false).as_bytes());
        key_info.extend_from_slice(keyid);
        let mut ikm = [0u8; 32];
        Hkdf::<Sha256>::new(Some(auth), shared.raw_secret_bytes().as_slice())
            .expand(&key_info, &mut ikm)
            .unwrap();
        let hk = Hkdf::<Sha256>::new(Some(salt), &ikm);
        let mut cek = [0u8; 16];
        hk.expand(b"Content-Encoding: aes128gcm\0", &mut cek)
            .unwrap();
        let mut nonce = [0u8; 12];
        hk.expand(b"Content-Encoding: nonce\0", &mut nonce).unwrap();
        let cipher = aes_gcm::Aes128Gcm::new_from_slice(&cek).unwrap();
        let mut plain = cipher.decrypt(&nonce.into(), record).unwrap();
        while plain.last() == Some(&0) {
            plain.pop();
        }
        assert_eq!(plain.pop(), Some(2), "last-record delimiter");
        plain
    }

    struct Vapid {
        header: serde_json::Value,
        claims: serde_json::Value,
        key: String,
    }

    /// RFC 8292 §3: `Authorization: vapid t=<JWT>, k=<public key>`; the JWT is
    /// ES256 (raw r||s) and must verify with `k`.
    fn verify_vapid(req: &axum::http::Request<Vec<u8>>) -> Vapid {
        use p256::ecdsa::signature::Verifier;
        let value = req.headers()[header::AUTHORIZATION].to_str().unwrap();
        let params = value.strip_prefix("vapid ").expect("vapid scheme");
        let mut token = None;
        let mut key = None;
        for part in params.split(',') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("t=") {
                token = Some(v);
            } else if let Some(v) = part.strip_prefix("k=") {
                key = Some(v);
            }
        }
        let (token, key) = (token.expect("t="), key.expect("k="));
        let (signing_input, sig) = token.rsplit_once('.').unwrap();
        let (head, claims) = signing_input.split_once('.').unwrap();
        let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(&b64(key)).unwrap();
        let sig = p256::ecdsa::Signature::from_slice(&b64(sig)).unwrap();
        vk.verify(signing_input.as_bytes(), &sig)
            .expect("VAPID JWT verifies with k");
        Vapid {
            header: serde_json::from_slice(&b64(head)).unwrap(),
            claims: serde_json::from_slice(&b64(claims)).unwrap(),
            key: key.to_string(),
        }
    }

    fn now_secs() -> i64 {
        chrono::Utc::now().timestamp()
    }

    fn auth_b64(byte: u8) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([byte; 16])
    }

    #[test]
    fn wake_request_decrypts_to_the_marker_with_the_subscriber_key() {
        let (vapid_public, scalar) = generate_vapid_keypair().unwrap();
        let (ua_secret, ua_public) = ua_keypair();
        let req = build_wake_request(
            "https://push.example/sub/1",
            &ua_public,
            &auth_b64(9),
            &scalar,
            CONTACT,
            WAKE_MARKER,
        )
        .unwrap();
        assert_eq!(req.method(), axum::http::Method::POST);
        assert_eq!(req.uri(), "https://push.example/sub/1");
        assert_eq!(req.headers()[header::CONTENT_ENCODING], "aes128gcm");
        assert_eq!(req.headers()["TTL"], WAKE_TTL.to_string().as_str());
        assert_eq!(
            decrypt_wake(req.body(), &ua_secret, &[9u8; 16]),
            WAKE_MARKER
        );
        // The JWT is signed with the stored VAPID key, and `k` is its public half.
        assert_eq!(verify_vapid(&req).key, vapid_public);
    }

    #[test]
    fn vapid_jwt_is_es256_and_expires_within_24_hours() {
        // RFC 8292 §2: "An exp claim MUST NOT be more than 24 hours from the time of
        // the request." The 4-week TTL header is a separate thing.
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        let before = now_secs();
        let req = build_wake_request(
            "https://push.example/sub/1",
            &ua_public,
            &auth_b64(1),
            &scalar,
            CONTACT,
            WAKE_MARKER,
        )
        .unwrap();
        let vapid = verify_vapid(&req);
        assert_eq!(vapid.header["alg"], "ES256");
        assert_eq!(vapid.header["typ"], "JWT");
        assert_eq!(vapid.claims["sub"], CONTACT);
        let exp = vapid.claims["exp"].as_i64().expect("numeric exp");
        assert!(exp > before, "exp in the future");
        assert!(
            exp <= now_secs() + 24 * 60 * 60,
            "exp {exp} is more than 24 h after {before}"
        );
    }

    #[test]
    fn vapid_audience_is_the_endpoint_origin_including_a_port() {
        // RFC 8292 §2: aud is the origin of the push resource URL (RFC 6454
        // serialisation: scheme, host, and the port when it is not the default).
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        for (endpoint, aud) in [
            ("https://push.example/wake/abc", "https://push.example"),
            ("https://push.example:443/wake/abc", "https://push.example"),
            (
                "https://push.example:8443/wake/abc",
                "https://push.example:8443",
            ),
            ("http://127.0.0.1:40850/push/1", "http://127.0.0.1:40850"),
        ] {
            let req = build_wake_request(
                endpoint,
                &ua_public,
                &auth_b64(2),
                &scalar,
                CONTACT,
                WAKE_MARKER,
            )
            .unwrap();
            assert_eq!(verify_vapid(&req).claims["aud"], aud, "{endpoint}");
        }
    }

    #[test]
    fn wake_request_refuses_bad_subscription_keys() {
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        let short_auth = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([3u8; 8]);
        let ep = "https://push.example/x";
        let a = auth_b64(3);
        assert!(build_wake_request(ep, "not-a-point", &a, &scalar, CONTACT, b"x").is_err());
        assert!(build_wake_request(ep, &ua_public, &short_auth, &scalar, CONTACT, b"x").is_err());
        assert!(build_wake_request(ep, &ua_public, &a, &[0u8; 32], CONTACT, b"x").is_err());
        assert!(build_wake_request("not a uri", &ua_public, &a, &scalar, CONTACT, b"x").is_err());
        assert!(
            build_wake_request(
                "ftp://push.example/x",
                &ua_public,
                &a,
                &scalar,
                CONTACT,
                b"x"
            )
            .is_err()
        );
        assert!(build_wake_request("/relative", &ua_public, &a, &scalar, CONTACT, b"x").is_err());
    }
}
